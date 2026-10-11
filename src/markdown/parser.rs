/*
 * Copyright (c) 2025 Jan Range
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 *
 */

use colored::Colorize;
use lazy_static::lazy_static;
use log::error;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::path::Path;

use pulldown_cmark::{CowStr, Event, HeadingLevel, OffsetIter, Options, Parser, Tag, TagEnd};
use regex::Regex;

use crate::attribute;
use crate::datamodel::DataModel;
use crate::object::{self, type_name, Enumeration, Object};
use crate::option::RawOption;
use crate::validation::{ErrorType, ValidationError, Validator};

use super::frontmatter::{parse_frontmatter, FrontMatter, ImportType};
use super::position::{Position, PositionRange};

lazy_static! {
    static ref MD_MODEL_TYPES: BTreeMap<&'static str, &'static str> = {
        let mut m = BTreeMap::new();
        m.insert(
            "Equation",
            include_str!("../../types/equation/equation-internal.json"),
        );
        m.insert(
            "UnitDefinition",
            include_str!("../../types/unit-definition/unit-definition-internal.json"),
        );
        m
    };
}

// Heading levels for re-use
const H1: Tag = Tag::Heading {
    level: HeadingLevel::H1,
    id: None,
    classes: Vec::new(),
    attrs: Vec::new(),
};
const H2: Tag = Tag::Heading {
    level: HeadingLevel::H2,
    id: None,
    classes: Vec::new(),
    attrs: Vec::new(),
};
const H3: Tag = Tag::Heading {
    level: HeadingLevel::H3,
    id: None,
    classes: Vec::new(),
    attrs: Vec::new(),
};

const H3_END: TagEnd = TagEnd::Heading(HeadingLevel::H3);

/// Mutable state carried through the object-parsing pass. `pending_heading`
/// holds the most recent heading that cannot open an object.
struct ParserContext {
    state: ParserState,
    errors: Vec<ValidationError>,
    pending_heading: Option<(usize, String, Position)>,
}

impl Default for ParserContext {
    fn default() -> Self {
        Self {
            state: ParserState::OutsideDefinition,
            errors: Vec::new(),
            pending_heading: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ParserState {
    InDefinition,
    OutsideDefinition,
    InHeading,
}

/// Parses a Markdown file and returns a DataModel.
///
/// # Arguments
/// * `content` - The markdown content to parse
/// * `path` - Optional path to the markdown file
///
/// # Returns
/// * `Result<DataModel, Validator>` - The parsed data model or validation errors
#[allow(clippy::result_large_err)]
pub fn parse_markdown(content: &str, path: Option<&Path>) -> Result<DataModel, Validator> {
    let content = clean_content(content);
    let config = parse_frontmatter(&content).unwrap_or_default();
    let line_offsets = create_line_offsets(&content);

    let mut model = DataModel::new(None, Some(config.clone()));
    let mut context = ParserContext::default();
    let (objects, enums) =
        parse_model_components(&content, &line_offsets, &mut model, &mut context);

    process_model_components(&mut model, objects, enums, &config, &mut context.errors);
    merge_imports(&mut model, config.imports, path, &mut context.errors);

    validate_model(&model, context.errors)?;
    Ok(model)
}

/// Creates a vector of line offset positions from content.
///
/// # Arguments
/// * `content` - The content to analyze
///
/// # Returns
/// * Vector of line offset positions
fn create_line_offsets(content: &str) -> Vec<usize> {
    content
        .char_indices()
        .filter(|(_, c)| *c == '\n')
        .map(|(i, _)| i)
        .collect()
}

/// Parses objects and enums from the markdown content.
///
/// # Arguments
/// * `content` - The markdown content to parse
/// * `line_offsets` - Vector of line offset positions
/// * `model` - Mutable reference to the data model
///
/// # Returns
/// * Tuple containing vectors of objects and enums
fn parse_model_components(
    content: &str,
    line_offsets: &[usize],
    model: &mut DataModel,
    context: &mut ParserContext,
) -> (Vec<Object>, Vec<Enumeration>) {
    let mut objects = Vec::new();
    let mut enums = Vec::new();

    // Parse objects
    let mut options = Options::empty();
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    let mut iterator = Parser::new_ext(content, options).into_offset_iter();

    while let Some(event) = iterator.next() {
        process_object_event(
            content,
            &mut iterator,
            &mut objects,
            event,
            model,
            line_offsets,
            context,
        );
    }

    // Parse enums
    let mut iterator = Parser::new(content).into_offset_iter();
    while let Some((event, range)) = iterator.next() {
        process_enum_event(
            content,
            &mut iterator,
            &mut enums,
            (event, range),
            line_offsets,
        );
    }

    (objects, enums)
}

/// Processes and filters model components, applying inheritance and internal types.
///
/// # Arguments
/// * `model` - Mutable reference to the data model
/// * `objects` - Vector of parsed objects
/// * `enums` - Vector of parsed enums
/// * `config` - Reference to the configuration
fn process_model_components(
    model: &mut DataModel,
    objects: Vec<Object>,
    enums: Vec<Enumeration>,
    config: &FrontMatter,
    errors: &mut Vec<ValidationError>,
) {
    let allow_empty = &config.allow_empty;

    // Filter and set components
    model.enums = enums.into_iter().filter(|e| e.has_values()).collect();
    model.objects = objects
        .into_iter()
        .filter(|o| {
            if *allow_empty {
                !&model.enums.iter().any(|e| e.name == o.name)
            } else {
                o.has_attributes()
            }
        })
        .collect();

    set_enum_attributes(model);
    add_internal_types(model, errors);
    if let Err(error) = add_mixin_types(model) {
        errors.push(syntax_error(
            &format!("Could not apply inheritance: {error}"),
            "Check that every name in a '[Mixin]' heading refers to an object in this model.",
            ErrorType::ObjectError,
            None,
            None,
            Vec::new(),
        ));
    }
}

/// Merges imported models into the main model.
///
/// # Arguments
/// * `model` - Mutable reference to the data model
/// * `imports` - The imports configuration
/// * `path` - Optional path to the markdown file
fn merge_imports(
    model: &mut DataModel,
    imports: HashMap<String, ImportType>,
    path: Option<&Path>,
    errors: &mut Vec<ValidationError>,
) {
    for (prefix, import) in imports {
        match import.fetch(path) {
            Ok(model_to_merge) => {
                if let Err(conflicts) = model.merge(&model_to_merge) {
                    errors.push(syntax_error(
                        &format!(
                            "The import '{prefix}' redefines {} with a different shape.",
                            conflicts.join(", ")
                        ),
                        "Rename the conflicting definitions, or remove the import.",
                        ErrorType::DuplicateError,
                        None,
                        None,
                        Vec::new(),
                    ));
                }
            }
            Err(error) => errors.push(syntax_error(
                &format!("Could not read the import '{prefix}': {error}"),
                "Check the path or URL given under the frontmatter 'imports' key.",
                ErrorType::GlobalError,
                None,
                None,
                Vec::new(),
            )),
        }
    }
}

/// Validates the model and returns any validation errors.
///
/// # Arguments
/// * `model` - Reference to the data model to validate
///
/// # Returns
/// * `Result<(), Validator>` - Ok if valid, Err with validator if invalid
#[allow(clippy::result_large_err)]
pub(crate) fn validate_model(
    model: &DataModel,
    parse_errors: Vec<ValidationError>,
) -> Result<(), Validator> {
    let mut validator = Validator::new();
    validator.validate(model);

    // `Validator::validate` clears its error list, so diagnostics collected
    // during parsing have to be added after it has run.
    for error in parse_errors {
        validator.add_error(error);
    }

    if !validator.is_valid {
        return Err(validator);
    }
    Ok(())
}

fn clean_content(content: &str) -> String {
    // Remove all html tags
    let re = Regex::new(r"<[^>]*>").unwrap();
    let content = re.replace_all(content, "").to_string();

    // Remove all Markdown links
    let re = Regex::new(r"\[([^]]+)]\([^)]+\)").unwrap();
    let content = re.replace_all(content.as_str(), "$1").to_string();

    content
}

// Helper function to convert byte offset to line and column numbers
fn get_position(content: &str, line_offsets: &[usize], start: usize, end: usize) -> Position {
    let line = match line_offsets.binary_search(&start) {
        Ok(line) => line + 1,
        Err(line) => line + 1,
    };

    // Get the line content
    let line_start = if line > 1 { line_offsets[line - 2] } else { 0 };
    let line_end = if line <= line_offsets.len() {
        line_offsets[line - 1]
    } else {
        content.len()
    };
    let line_content = &content[line_start..line_end];

    // Count leading whitespace
    let leading_space = line_content
        .chars()
        .take_while(|c| c.is_whitespace())
        .count();

    // Calculate column numbers, adding leading whitespace to start
    let start_col = if line > 1 {
        start - line_offsets[line - 2] + leading_space - 1
    } else {
        start + 1 + leading_space
    };

    let end_col = if line <= line_offsets.len() {
        line_offsets[line - 1] - (if line > 1 { line_offsets[line - 2] } else { 0 })
    } else {
        end - (if line > 1 { line_offsets[line - 2] } else { 0 })
    };

    Position {
        line,
        column: PositionRange {
            start: start_col,
            end: end_col,
        },
        offset: PositionRange { start, end },
    }
}

/// Processes a single Markdown event for object extraction.
///
/// # Arguments
///
/// * `content` - The full content of the markdown file
/// * `iterator` - A mutable reference to the parser iterator
/// * `objects` - A mutable reference to the vector of objects
/// * `event` - The current Markdown event and its range
/// * `model` - A mutable reference to the data model
/// * `state` - A mutable reference to the parser state
/// * `line_offsets` - A reference to the line offsets of the file
fn process_object_event(
    content: &str,
    iterator: &mut pulldown_cmark::OffsetIter,
    objects: &mut Vec<Object>,
    event: (Event, std::ops::Range<usize>),
    model: &mut DataModel,
    line_offsets: &[usize],
    context: &mut ParserContext,
) {
    let (event, range) = event;

    match event {
        // A level-1 heading names the model, so the prose and lists that follow
        // it are the model's preamble rather than a misplaced object.
        Event::Start(tag) if tag == H1 => {
            handle_h1_event(iterator, model);
            context.state = ParserState::OutsideDefinition;
            context.pending_heading = None;
        }
        Event::Start(tag) if tag == H2 => {
            context.state = ParserState::OutsideDefinition;
            context.pending_heading = Some((
                2,
                heading_text(content, &range),
                get_position(content, line_offsets, range.start, range.end),
            ));
        }
        Event::Start(tag) if tag == H3 => {
            context.pending_heading = None;
            handle_h3_start(content, iterator, objects, line_offsets, range, context);
        }
        Event::End(tag) if tag == H3_END => {
            context.state = ParserState::InDefinition;
        }
        // Only a level-3 heading opens an object. Headings below that level were
        // not handled at all, so their attribute lists were appended to whatever
        // object was open, while a list under a level-1 or level-2 heading was
        // dropped. Both are recorded so the model reports the problem.
        Event::Start(Tag::Heading { level, .. })
            if matches!(
                level,
                HeadingLevel::H4 | HeadingLevel::H5 | HeadingLevel::H6
            ) =>
        {
            context.state = ParserState::OutsideDefinition;
            context.pending_heading = Some((
                heading_level_number(level),
                heading_text(content, &range),
                get_position(content, line_offsets, range.start, range.end),
            ));
        }
        Event::Text(CowStr::Borrowed(text)) if text.starts_with(":") => {
            handle_type_annotation(objects, text);
        }
        Event::Text(CowStr::Borrowed("[")) => {
            if context.state == ParserState::InHeading {
                handle_mixin(objects, iterator, context);
            }
        }
        Event::Start(Tag::List(None)) => {
            if context.state == ParserState::OutsideDefinition {
                // The heading stays pending until a heading that can open an
                // object replaces it, so a narrative list before the real
                // attribute list does not consume the check.
                if looks_like_attribute_list(content, &range) {
                    if let Some((level, name, position)) = context.pending_heading.take() {
                        context
                            .errors
                            .push(misplaced_object_error(level, &name, position));
                    }
                }
                return;
            }

            handle_list_start(content, iterator, objects, line_offsets, range, context);
        }
        Event::Start(Tag::Item) => {
            if context.state == ParserState::OutsideDefinition {
                return;
            }

            handle_list_item(content, iterator, objects, line_offsets, range, context);
        }
        Event::Text(text) if text.to_string() == "]" => {
            handle_array_marker(objects);
        }
        Event::Text(text) if context.state == ParserState::InDefinition => {
            handle_docstring(objects, text);
        }
        _ => {}
    }
}

/// Builds a parse-time diagnostic naming the construct and its line.
fn syntax_error(
    message: &str,
    solution: &str,
    error_type: ErrorType,
    object: Option<String>,
    attribute: Option<String>,
    positions: Vec<Position>,
) -> ValidationError {
    ValidationError {
        message: message.to_string(),
        object,
        attribute,
        location: "Global".into(),
        error_type,
        solution: Some(solution.to_string()),
        positions,
    }
}

/// Numeric level of a heading, for diagnostics.
fn heading_level_number(level: HeadingLevel) -> usize {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// Reads a heading's text straight from the source.
fn heading_text(content: &str, range: &std::ops::Range<usize>) -> String {
    content
        .get(range.clone())
        .unwrap_or_default()
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .trim_start_matches('#')
        .trim()
        .to_string()
}

lazy_static! {
    /// Matches a sub-item that declares one of the attribute option keys, which
    /// is what distinguishes an object definition from a narrative bullet list.
    static ref ATTRIBUTE_OPTION_KEY: Regex = Regex::new(
        r"(?im)^\s*[-*+]\s*(type|term|description|xml|default|multiple|pattern|regex|minimum|maximum)\s*:"
    )
    .unwrap();
}

/// Decides whether a bullet list is an attempted object definition.
///
/// The test is structural rather than keyword-based, so narrative prose does
/// not match: either an item whose text is a plausible attribute name carrying
/// an indented option, or a shorthand `name: Type` with single-token sides.
fn looks_like_attribute_list(content: &str, range: &std::ops::Range<usize>) -> bool {
    let raw = content.get(range.clone()).unwrap_or_default();

    // Fenced code inside a list item is an example, not a declaration.
    let mut in_fence = false;
    let lines: Vec<&str> = raw
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_fence = !in_fence;
                return false;
            }
            !in_fence
        })
        .collect();

    let base_indent = lines
        .iter()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len());
    let Some(base_indent) = base_indent else {
        return false;
    };

    let mut item_is_candidate_name = false;
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let Some(text) = strip_list_marker(line.trim_start()) else {
            continue;
        };

        if indent <= base_indent {
            // A new top-level item: either a shorthand declaration on its own,
            // or a name whose options follow underneath.
            if is_shorthand_attribute(text) {
                return true;
            }
            item_is_candidate_name = is_attribute_name(text);
        } else if item_is_candidate_name && ATTRIBUTE_OPTION_KEY.is_match(line) {
            return true;
        }
    }

    false
}

/// Whether the text could be an attribute name: one token, optionally emboldened.
fn is_attribute_name(text: &str) -> bool {
    let name = text
        .trim()
        .trim_matches('*')
        .trim_matches('_')
        .trim_end_matches(':');
    !name.is_empty()
        && !name.contains(char::is_whitespace)
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

/// Whether the text is a shorthand declaration, `name: Type`.
fn is_shorthand_attribute(text: &str) -> bool {
    let Some((name, dtype)) = text.split_once(':') else {
        return false;
    };
    let dtype = dtype.trim().trim_end_matches("[]");
    is_attribute_name(name)
        && !dtype.is_empty()
        && !dtype.contains(char::is_whitespace)
        && dtype
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

/// Builds the diagnostic for an object defined at the wrong heading level.
fn misplaced_object_error(level: usize, name: &str, position: Position) -> ValidationError {
    ValidationError {
        message: format!(
            "'{name}' looks like an object definition but is written as a level-{level} heading, \
             so it is not part of the model."
        ),
        object: Some(name.to_string()),
        attribute: None,
        location: "Global".into(),
        error_type: ErrorType::ObjectError,
        solution: Some(format!(
            "Objects are declared with a level-3 heading. Write '### {name}' instead of \
             '{} {name}'.",
            "#".repeat(level)
        )),
        positions: vec![position],
    }
}

/// Handles H1 heading events by setting the model name.
///
/// # Arguments
///
/// * `iterator` - A mutable reference to the markdown parser iterator
/// * `model` - A mutable reference to the data model to update
fn handle_h1_event(iterator: &mut pulldown_cmark::OffsetIter, model: &mut DataModel) {
    model.name = extract_name(iterator);
}

/// Handles H3 heading start events by creating and adding a new object.
///
/// # Arguments
///
/// * `content` - The full content of the markdown file
/// * `iterator` - A mutable reference to the parser iterator
/// * `objects` - A mutable reference to the vector of objects
/// * `state` - A mutable reference to the parser state
/// * `line_offsets` - A reference to the line offsets of the file
/// * `range` - The byte range of the current event
fn handle_h3_start(
    content: &str,
    iterator: &mut pulldown_cmark::OffsetIter,
    objects: &mut Vec<Object>,
    line_offsets: &[usize],
    range: std::ops::Range<usize>,
    context: &mut ParserContext,
) {
    context.state = ParserState::InHeading;
    let position = get_position(content, line_offsets, range.start, range.end);

    let Some(mut object) = process_object_heading(iterator) else {
        context.errors.push(syntax_error(
            "This heading declares an object but has no readable name.",
            "Give the object a plain-text name, for example '### Experiment'.",
            ErrorType::NameError,
            None,
            None,
            vec![position],
        ));
        return;
    };

    object.set_position(position);
    objects.push(object);
}

/// Handles type annotations in the format ": type".
///
/// # Arguments
///
/// * `objects` - A mutable slice of objects
/// * `text` - The text containing the type annotation
fn handle_type_annotation(objects: &mut [Object], text: &str) {
    let Some(object) = objects.last_mut() else {
        return;
    };
    let attribute = object.get_last_attribute();
    if let Some(attribute) = attribute {
        attribute
            .add_option(RawOption::new(
                "type".to_string(),
                text.to_string().trim_start_matches(':').trim().to_string(),
            ))
            .unwrap();
    }
}

/// Handles inheritance declarations in object headings.
///
/// # Arguments
///
/// * `objects` - A mutable slice of objects
/// * `iterator` - A mutable reference to the parser iterator
fn handle_mixin(
    objects: &mut [Object],
    iterator: &mut pulldown_cmark::OffsetIter,
    context: &mut ParserContext,
) {
    let Some(last_object) = objects.last_mut() else {
        return;
    };
    let object_name = last_object.name.clone();
    let positions = last_object.position.map(|p| vec![p]).unwrap_or_default();
    let mixin = iterator.next();

    match mixin {
        Some((Event::Text(text), _)) if text.to_string() != "]" => {
            last_object.mixins = text.split(',').map(|s| s.trim().to_string()).collect();
        }
        _ => {
            // A hard error: skipping the mixin would leave a model that
            // validates while missing inherited properties.
            context.errors.push(syntax_error(
                "Opening bracket in the heading but no mixin name follows it.",
                "Name the object to inherit from, e.g. '### Child [Parent]', or remove the bracket.",
                ErrorType::ObjectError,
                Some(object_name),
                None,
                positions,
            ));
        }
    }
}

/// Handles the start of a list, processing either attributes or attribute options.
///
/// # Arguments
///
/// * `content` - The full content of the markdown file
/// * `iterator` - A mutable reference to the parser iterator
/// * `objects` - A mutable reference to the vector of objects
/// * `line_offsets` - A reference to the line offsets of the file
/// * `range` - The byte range of the current event
fn handle_list_start(
    content: &str,
    iterator: &mut pulldown_cmark::OffsetIter,
    objects: &mut [Object],
    line_offsets: &[usize],
    range: std::ops::Range<usize>,
    context: &mut ParserContext,
) {
    let Some(last_object) = objects.last_mut() else {
        return;
    };

    if !last_object.has_attributes() {
        iterator.next();
        let position = get_position(content, line_offsets, range.start, range.end);
        add_attribute_from_iterator(objects, iterator, position, context);
    } else {
        let attr_strings = extract_attribute_options(content, iterator);
        for attr_string in attr_strings {
            distribute_attribute_options(objects, attr_string, context);
        }
    }
}

/// Builds an attribute from the upcoming events and appends it to the last
/// object, reporting a diagnostic when the item carries no usable name.
fn add_attribute_from_iterator(
    objects: &mut [Object],
    iterator: &mut pulldown_cmark::OffsetIter,
    position: Position,
    context: &mut ParserContext,
) {
    let object_name = objects.last().map(|o| o.name.clone());

    let Some((required, attr_name, dtypes)) = extract_attr_name_required(iterator) else {
        context.errors.push(syntax_error(
            "This list item declares a property but has no readable name.",
            "Write the name as plain text, or in bold to mark it required, e.g. '- **name**'.",
            ErrorType::NameError,
            object_name,
            None,
            vec![position],
        ));
        return;
    };

    let mut attribute = attribute::Attribute::new(attr_name.clone(), required);

    if let Some((key, dtypes)) = dtypes {
        if let Err(error) = attribute.add_option(RawOption::new(key, dtypes)) {
            context.errors.push(syntax_error(
                &format!("Could not read the type of '{attr_name}': {error}"),
                "Check the 'Type:' declaration for this property.",
                ErrorType::TypeError,
                object_name,
                Some(attr_name),
                vec![position],
            ));
            return;
        }
    }

    attribute.set_position(position);
    if let Some(object) = objects.last_mut() {
        object.add_attribute(attribute);
    }
}

/// Handles list items by creating new attributes.
///
/// # Arguments
///
/// * `content` - The full content of the markdown file
/// * `iterator` - A mutable reference to the parser iterator
/// * `objects` - A mutable reference to the vector of objects
/// * `line_offsets` - A reference to the line offsets of the file
/// * `range` - The byte range of the current event
fn handle_list_item(
    content: &str,
    iterator: &mut pulldown_cmark::OffsetIter,
    objects: &mut [Object],
    line_offsets: &[usize],
    range: std::ops::Range<usize>,
    context: &mut ParserContext,
) {
    let position = get_position(content, line_offsets, range.start, range.end);
    add_attribute_from_iterator(objects, iterator, position, context);
}

/// Handles array markers by setting the is_array flag on the last attribute.
///
/// # Arguments
///
/// * `objects` - A mutable slice of objects
fn handle_array_marker(objects: &mut [Object]) {
    let Some(last_object) = objects.last_mut() else {
        return;
    };
    let last_attribute = last_object.get_last_attribute();
    if let Some(attribute) = last_attribute {
        attribute.is_array = true;
    }
}

/// Handles docstring text by appending it to the last object's docstring.
///
/// # Arguments
///
/// * `objects` - A mutable slice of objects
/// * `text` - The text to append to the docstring
fn handle_docstring(objects: &mut [Object], text: CowStr) {
    let Some(last_object) = objects.last_mut() else {
        return;
    };
    if !last_object.docstring.is_empty() {
        last_object
            .docstring
            .push_str(format!(" {}", text.as_ref()).as_str());
    } else {
        last_object.docstring = text.as_ref().to_string();
    }

    // Remove all spaces > 1
    last_object.docstring = last_object
        .docstring
        .split_whitespace()
        .map(|s| s.trim())
        .collect::<Vec<&str>>()
        .join(" ");
}

/// Processes the heading of an object.
///
/// # Arguments
///
/// * `iterator` - A mutable reference to the parser iterator.
///
/// # Returns
///
/// An `Object` created from the heading.
fn process_object_heading(iterator: &mut OffsetIter) -> Option<object::Object> {
    let heading = extract_name(iterator)?;
    let (cleaned_name, term) = extract_object_term(&heading);
    // Use the cleaned name (Object::new will convert it to PascalCase)
    Some(object::Object::new(cleaned_name, term))
}

/// Extracts the name from the next text event in the iterator.
///
/// # Arguments
///
/// * `iterator` - A mutable reference to the parser iterator.
///
/// # Returns
///
/// `Some(name)`, or `None` when the heading carries no plain text — an empty
/// heading, or one written entirely as a code span or link.
fn extract_name(iterator: &mut OffsetIter) -> Option<String> {
    if let Some((Event::Text(text), _)) = iterator.next() {
        return Some(text.to_string());
    }

    // Try for two text events
    for _ in 0..2 {
        if let Some((Event::Text(text), _)) = iterator.next() {
            return Some(text.to_string());
        }
    }

    None
}

/// Extracts the attribute name and its required status from the iterator.
///
/// # Arguments
///
/// * `iterator` - A mutable reference to the parser iterator.
///
/// # Returns
///
/// A tuple containing a boolean indicating if the attribute is required and the attribute name.
#[allow(clippy::type_complexity)]
fn extract_attr_name_required(
    iterator: &mut OffsetIter,
) -> Option<(bool, String, Option<(String, String)>)> {
    let mut next = iterator.next();

    // If there are newlines between the attributes, the likely a paragraph
    // is being started. We need to consume the paragraph and the following
    // text event.
    if let Some((Event::Start(Tag::Paragraph), _)) = next {
        next = iterator.next();
    }

    match next {
        Some((Event::Text(text), _)) => {
            if let Some((key, dtypes)) = shorthand_type(&text) {
                return Some((false, key, Some(dtypes)));
            } else {
                return Some((false, text.to_string(), None));
            }
        }
        Some((Event::Start(Tag::Strong), _)) => {
            let next = iterator.next();
            let mut name = String::new();
            if let Some((Event::Text(text), _)) = next {
                name = text.to_string();
            }

            // Consume the Strong end tag
            iterator.next();

            if name.is_empty() {
                return None;
            }

            return Some((true, name, None));
        }
        _ => {}
    }

    // Anything else — an empty item, or a name written as italics or a code
    // span rather than the bold-means-required convention.
    None
}

/// Extracts a type definition from shorthand notation in the form "key: type".
///
/// # Arguments
///
/// * `text` - A string slice containing the potential shorthand type definition
///
/// # Returns
///
/// An `Option` containing a tuple of `(key, type)` strings if the text matches the shorthand format,
/// or `None` if it does not contain a colon separator.
fn shorthand_type(text: &str) -> Option<(String, (String, String))> {
    if let Some((key, dtypes)) = text.split_once(":") {
        Some((
            key.trim().to_string(),
            ("type".to_string(), dtypes.trim().to_string()),
        ))
    } else {
        None
    }
}

/// Extracts the term and cleaned name from an object heading.
///
/// # Arguments
///
/// * `heading` - A string slice containing the heading.
///
/// # Returns
///
/// A tuple containing:
/// - First element: cleaned name (heading with parentheses removed and trailing whitespace trimmed)
/// - Second element: optional term extracted from inside parentheses
fn extract_object_term(heading: &str) -> (String, Option<String>) {
    // Find the last occurrence of parentheses to handle nested cases
    if let Some(start) = heading.rfind('(') {
        if let Some(end) = heading[start..].find(')') {
            let term = heading[start + 1..start + end].to_string();
            let cleaned_name = heading[..start].trim_end().to_string();
            return (cleaned_name, Some(term));
        }
    }

    // No parentheses found, return the heading trimmed
    (heading.trim_end().to_string(), None)
}

/// Extracts attribute options from the iterator.
///
/// # Arguments
///
/// * `iterator` - A mutable reference to the parser iterator.
///
/// # Returns
///
/// A vector of strings containing the extracted attribute options.
fn extract_attribute_options(content: &str, iterator: &mut OffsetIter) -> Vec<String> {
    let mut options = Vec::new();
    for (next, range) in iterator.by_ref() {
        match next {
            Event::Start(Tag::Item) => {
                options.push(raw_list_item_value(content, range));
            }
            Event::End(TagEnd::List(false)) => {
                break;
            }
            _ => {}
        }
    }
    options
}

/// Reads a list item's value from the source text.
///
/// Option values are literals, so they are not rebuilt from the inline event
/// stream, which consumes `_` and `*` as emphasis. Continuation lines are
/// joined with a single space; spacing within a line is kept verbatim.
fn raw_list_item_value(content: &str, range: std::ops::Range<usize>) -> String {
    let raw = content.get(range).unwrap_or_default();

    // The reported range for a nested item starts at the newline that precedes
    // it, so the first line of the slice can be empty. Skip forward to the line
    // that actually carries the list marker.
    let mut lines = raw.lines().skip_while(|line| line.trim().is_empty());

    let head = lines.next().unwrap_or_default().trim_start();
    // Strip the bullet marker if there is one. An item reported without a
    // marker (an ordered sub-list, for instance) must keep its text: dropping
    // it yields an empty attribute name, which downstream code indexes into.
    let first = match strip_list_marker(head) {
        Some(rest) => rest,
        None => head,
    };

    let mut value = String::from(first.trim_end());
    for line in lines {
        let line = line.trim();
        // A nested list is a separate construct. CommonMark only opens a list
        // when the marker is followed by whitespace, so `-10` stays as text.
        if line.is_empty() || strip_list_marker(line).is_some() {
            continue;
        }
        if !value.is_empty() {
            value.push(' ');
        }
        value.push_str(line);
    }

    unescape_markdown(&value)
}

/// Resolves CommonMark backslash escapes in a literal read from the source.
fn unescape_markdown(value: &str) -> String {
    const ESCAPABLE: &str = r##"!"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~"##;

    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                if ESCAPABLE.contains(next) {
                    out.push(next);
                    chars.next();
                    continue;
                }
            }
        }
        out.push(c);
    }
    out
}

/// Strips a CommonMark bullet marker. Returns `None` when the marker is not
/// followed by whitespace, so `-10` and `*emphasised*` stay as text.
fn strip_list_marker(line: &str) -> Option<&str> {
    let rest = line.strip_prefix(['-', '*', '+'])?;
    if rest.is_empty() {
        return Some(rest);
    }
    if rest.starts_with([' ', '\t']) {
        return Some(rest.trim_start());
    }
    None
}

/// Adds an option to the last attribute of the last object in the list.
///
/// # Arguments
///
/// * `objects` - A mutable reference to the list of objects.
/// * `key` - The key of the attribute option.
/// * `value` - The value of the attribute option.
fn add_option_to_last_attribute(
    objects: &mut [object::Object],
    key: String,
    value: String,
) -> Result<(), Box<dyn Error>> {
    let Some(object) = objects.last_mut() else {
        return Ok(());
    };
    let last_attr = object.get_last_attribute();
    if let Some(attribute) = last_attr {
        let option = RawOption::new(key, value);
        attribute.add_option(option)?;
    }

    Ok(())
}

/// Distributes attribute options among the objects.
///
/// # Arguments
///
/// * `objects` - A mutable reference to the list of objects.
/// * `attr_string` - A string containing the attribute or option.
///
/// # Returns
///
/// An optional unit type.
fn distribute_attribute_options(
    objects: &mut [object::Object],
    attr_string: String,
    context: &mut ParserContext,
) -> Option<()> {
    if attr_string.contains(':') {
        let (key, value) = process_option(&attr_string);
        let object_name = objects.last().map(|o| o.name.clone());
        let attribute_name = objects
            .last()
            .and_then(|o| o.attributes.last())
            .map(|a| a.name.clone());
        let positions = objects
            .last()
            .and_then(|o| o.attributes.last())
            .and_then(|a| a.position)
            .map(|p| vec![p])
            .unwrap_or_default();

        if let Err(error) = add_option_to_last_attribute(objects, key.clone(), value) {
            context.errors.push(syntax_error(
                &format!("Could not read the option '{key}': {error}"),
                "Check the value given for this option.",
                ErrorType::TypeError,
                object_name,
                attribute_name,
                positions,
            ));
        }
        return None;
    }

    objects
        .last_mut()
        .unwrap()
        .create_new_attribute(attr_string, false);

    None
}

/// Processes an attribute option string.
///
/// # Arguments
///
/// * `option` - A string containing the attribute option.
///
/// # Returns
///
/// A tuple containing the key and value of the attribute option.
fn process_option(option: &String) -> (String, String) {
    let parts: Vec<&str> = option.split(':').collect();

    assert!(
        parts.len() > 1,
        "Attribute {option} does not have a valid option"
    );

    let key = parts[0].trim();
    let value = parts[1..].join(":");

    (key.to_string(), value.trim().to_string())
}

/// Processes a single Markdown event for enumeration extraction.
///
/// # Arguments
///
/// * `iterator` - A mutable reference to the parser iterator.
/// * `enums` - A mutable reference to the vector of enumerations.
/// * `event` - The current Markdown event.
/// * `range` - The range of the event.
/// * `line_offsets` - The line offsets of the file.
pub fn process_enum_event(
    content: &str,
    iterator: &mut OffsetIter,
    enums: &mut Vec<Enumeration>,
    event: (Event, std::ops::Range<usize>),
    line_offsets: &[usize],
) {
    let (event, range) = event;

    match event {
        Event::Start(tag) if tag == H3 => {
            let Some(enum_name) = extract_name(iterator) else {
                // An enumeration heading with no readable name; the object pass
                // reports it, so it is simply skipped here.
                return;
            };
            let mut enum_obj = Enumeration {
                name: type_name(&enum_name),
                mappings: BTreeMap::new(),
                docstring: "".to_string(),
                position: None,
            };
            enum_obj.set_position(get_position(content, line_offsets, range.start, range.end));
            enums.push(enum_obj);
        }
        Event::Start(Tag::CodeBlock(pulldown_cmark::CodeBlockKind::Fenced(_))) => {
            let event = iterator.next().unwrap();
            if let (Event::Text(text), _) = event {
                let mappings = text.to_string();

                if let Some(enum_obj) = enums.last_mut() {
                    process_enum_mappings(enum_obj, mappings);
                }
            }
        }
        _ => {}
    }
}

/// Processes enumeration mappings from a code block.
///
/// # Arguments
///
/// * `enum_obj` - A mutable reference to the enumeration object.
/// * `mappings` - A string containing the mappings.
fn process_enum_mappings(enum_obj: &mut Enumeration, mappings: String) {
    let lines = mappings.split('\n');
    for line in lines {
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() != 2 {
            // Skip empty lines or lines that do not contain a mapping
            continue;
        }

        // Extract key and value, insert into enum object
        let key = parts[0].trim().replace('"', "");
        let value = parts[1].trim().replace('"', "");
        enum_obj.mappings.insert(key.to_string(), value.to_string());
    }
}

/// Adds mixin types to the objects in the model.
///
/// This function processes mixins by adding attributes from mixin objects to their children.
/// It handles both user-defined mixins and internal type mixins.
///
/// # Arguments
///
/// * `model` - A mutable reference to the data model.
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - Ok if successful, or an error if mixin fails.
///
/// # Panics
///
/// Panics if an object has a mixin that does not exist.
///
/// # Errors
///
/// An error is logged if an object has a mixin that does not exist.
///
fn add_mixin_types(model: &mut DataModel) -> Result<(), Box<dyn Error>> {
    // Filter and clone the objects without a mixin
    let mixins = collect_mixin_objects(model);

    let mut to_merge = Vec::new();
    let mut added_internals = Vec::new();

    // Process each object that has a mixin
    for object in model.objects.iter_mut() {
        for local_mixin in object.mixins.clone() {
            process_mixins(
                object,
                &local_mixin,
                &mixins,
                &mut to_merge,
                &mut added_internals,
            )?;
        }
    }

    // Merge all collected internal types
    merge_internal_types(model, to_merge);

    Ok(())
}

/// Collects all objects that can serve as mixins (those without mixins themselves)
///
/// # Arguments
///
/// * `model` - A reference to the data model.
///
/// # Returns
///
/// * `Vec<Object>` - A vector of objects that can be used as mixins.
fn collect_mixin_objects(model: &DataModel) -> Vec<Object> {
    model
        .objects
        .iter()
        .filter(|o| o.mixins.is_empty())
        .cloned()
        .collect()
}

/// Processes mixin for a single object
///
/// This function handles the mixin logic for a specific object by finding its mixin
/// and extending the object's attributes with those from the mixin.
///
/// # Arguments
///
/// * `object` - A mutable reference to the object being processed.
/// * `mixin_name` - The name of the mixin object.
/// * `mixins` - A slice of available mixin objects.
/// * `to_merge` - A mutable reference to a vector of data models to be merged.
/// * `added_internals` - A mutable reference to a vector of internal type names that have been added.
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - Ok if successful, or an error if the mixin is not found.
fn process_mixins(
    object: &mut Object,
    mixin_name: &str,
    mixins: &[Object],
    to_merge: &mut Vec<DataModel>,
    added_internals: &mut Vec<String>,
) -> Result<(), Box<dyn Error>> {
    if let Some(mixin) = find_mixin_in_objects(mixin_name, mixins) {
        // Mixin found in existing objects
        object.attributes.extend(mixin.attributes.clone());
    } else if let Some(internal_type) = MD_MODEL_TYPES.get(mixin_name) {
        // Mixin is an internal type
        process_internal_mixin_type(object, mixin_name, internal_type, to_merge, added_internals);
    } else {
        // Mixin not found
        return report_missing_mixin(object, mixin_name);
    }

    Ok(())
}

/// Finds a mixin object by name in the list of available mixins
///
/// # Arguments
///
/// * `mixin_name` - The name of the mixin object to find.
/// * `mixins` - A slice of available mixin objects.
///
/// # Returns
///
/// * `Option<&'a Object>` - A reference to the mixin object if found, or None if not found.
fn find_mixin_in_objects<'a>(mixin_name: &str, mixins: &'a [Object]) -> Option<&'a Object> {
    mixins.iter().find(|o| o.name == mixin_name)
}

/// Processes an internal mixin type
///
/// This function handles inheritance from internal predefined types by extracting
/// attributes from the internal type and adding them to the object.
///
/// # Arguments
///
/// * `object` - A mutable reference to the object being processed.
/// * `mixin_name` - The name of the internal mixin type.
/// * `internal_type_json` - The JSON string representation of the internal type.
/// * `to_merge` - A mutable reference to a vector of data models to be merged.
/// * `added_internals` - A mutable reference to a vector of internal type names that have been added.
fn process_internal_mixin_type(
    object: &mut Object,
    mixin_name: &str,
    internal_type_json: &str,
    to_merge: &mut Vec<DataModel>,
    added_internals: &mut Vec<String>,
) {
    let mut internal_type = serde_json::from_str::<DataModel>(internal_type_json)
        .expect("Failed to parse internal data type");

    // Pop the first object mixin and add the attributes
    let target_obj = internal_type.objects[0].clone();
    internal_type.objects.remove(0);

    object.attributes.extend(target_obj.attributes.clone());

    if !added_internals.contains(&mixin_name.to_string()) {
        to_merge.push(internal_type);
        added_internals.push(mixin_name.to_string());
    }
}

/// Reports a missing mixin error
///
/// # Arguments
///
/// * `object` - A reference to the object with the missing mixin.
/// * `mixin_name` - The name of the mixin that was not found.
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - An error indicating the mixin does not exist.
fn report_missing_mixin(object: &Object, mixin_name: &str) -> Result<(), Box<dyn Error>> {
    error!(
        "[{}] {}: Mixin {} does not exist.",
        object.name.red().bold(),
        "InheritanceError".bold(),
        mixin_name.red().bold(),
    );

    Err("Object has a mixin that does not exist".into())
}

/// Merges all collected internal types into the model
///
/// # Arguments
///
/// * `model` - A mutable reference to the data model.
/// * `to_merge` - A vector of data models to be merged into the main model.
fn merge_internal_types(model: &mut DataModel, to_merge: Vec<DataModel>) {
    for internal in to_merge {
        let _ = model.merge(&internal);
    }
}

/// Adds internal types to the model based on attribute data types
///
/// This function identifies internal types that are referenced in object attributes
/// and adds them to the model if they're not already present.
///
/// # Arguments
///
/// * `model` - A mutable reference to the data model.
fn add_internal_types(model: &mut DataModel, errors: &mut Vec<ValidationError>) {
    // Get all datatypes within the model
    let mut all_types = vec![];
    for object in &model.objects {
        for attr in &object.attributes {
            all_types.extend(attr.dtypes.clone());
        }
    }

    let object_names = model
        .objects
        .iter()
        .map(|obj| obj.name.clone())
        .collect::<Vec<String>>();

    for (name, content) in MD_MODEL_TYPES.iter() {
        if object_names.contains(&name.to_string()) {
            continue;
        }

        if all_types.contains(&name.to_string()) {
            let internal = match serde_json::from_str::<DataModel>(content) {
                Ok(internal) => internal,
                Err(error) => {
                    errors.push(syntax_error(
                        &format!("The built-in type '{name}' could not be loaded: {error}"),
                        "This is an internal error; please report it.",
                        ErrorType::GlobalError,
                        None,
                        None,
                        Vec::new(),
                    ));
                    continue;
                }
            };

            // A user object whose name collides with one inside a built-in type
            // is an authoring mistake, not a reason to abort.
            if let Err(conflicts) = model.merge(&internal) {
                errors.push(syntax_error(
                    &format!(
                        "Using the built-in type '{name}' requires {}, which this model already \
                         defines differently.",
                        conflicts.join(", ")
                    ),
                    &format!(
                        "Rename {} , or stop referencing '{name}'.",
                        conflicts.join(", ")
                    ),
                    ErrorType::DuplicateError,
                    None,
                    None,
                    Vec::new(),
                ));
            }
        }
    }
}

/// Sets the `is_enum` flag for attributes that are enumerations.
///
/// This function iterates through all objects and their attributes in the data model.
/// If an attribute's data types match any of the enumeration names, the `is_enum` flag
/// is set to `true`. If an attribute has data types that do not match any enumeration,
/// an error is returned.
///
/// # Arguments
///
/// * `model` - A mutable reference to the data model.
fn set_enum_attributes(model: &mut DataModel) {
    let enums = model
        .enums
        .iter()
        .map(|e| e.name.clone())
        .collect::<Vec<String>>();

    for object in model.objects.iter_mut() {
        for attr in object.attributes.iter_mut() {
            let enum_dtypes: Vec<String> = attr
                .dtypes
                .iter()
                .filter(|dtype| enums.contains(dtype))
                .cloned()
                .collect();
            if !enum_dtypes.is_empty() && enum_dtypes.len() == attr.dtypes.len() {
                attr.is_enum = true;
            }
        }
    }
}

/// Represents the different keys that can be used for attribute options.
pub(crate) enum OptionKey {
    /// Represents the data type of the attribute.
    Type,
    /// Represents the term associated with the attribute.
    Term,
    /// Represents the description of the attribute.
    Description,
    /// Represents the XML type information for the attribute.
    Xml,
    /// Represents the default value for the attribute.
    Default,
    /// Indicates if the attribute can have multiple values.
    Multiple,
    /// Represents any other option not covered by the predefined keys.
    Other,
}

impl OptionKey {
    /// Converts a string to an `OptionKey`.
    ///
    /// # Arguments
    ///
    /// * `key` - The string representation of the key.
    ///
    /// # Returns
    ///
    /// An `OptionKey` corresponding to the given string.
    pub fn from_str(key: &str) -> Self {
        match key.to_lowercase().as_str() {
            "type" => OptionKey::Type,
            "term" => OptionKey::Term,
            "description" => OptionKey::Description,
            "xml" => OptionKey::Xml,
            "default" => OptionKey::Default,
            "multiple" => OptionKey::Multiple,
            _ => OptionKey::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::option::AttrOption;
    use pretty_assertions::assert_eq;

    /// Wraps an option line in a minimal model and returns the parsed model.
    fn parse_with_option(option_line: &str) -> DataModel {
        let content = format!(
            "---\nprefix: \"t\"\nrepo: \"https://e.org/\"\n---\n\n### P\n\n- a\n  - Type: string\n  {option_line}\n"
        );
        parse_markdown(&content, None).expect("model should parse")
    }

    /// Returns the `Pattern` option value of the first attribute, if any.
    fn pattern_of(option_line: &str) -> Option<String> {
        parse_with_option(option_line).objects[0].attributes[0]
            .options
            .iter()
            .find_map(|o| match o {
                AttrOption::Pattern(p) => Some(p.clone()),
                _ => None,
            })
    }

    //
    // Option values are literals and must survive the parser byte for byte.
    //
    // The inline event stream consumes `_` and `*` as emphasis and drops the
    // whitespace next to them, which would rewrite a regular expression.
    //

    #[test]
    fn test_pattern_preserves_space_in_character_class() {
        assert_eq!(
            pattern_of("- Pattern: ^[A-Za-z0-9 _-]+$").as_deref(),
            Some("^[A-Za-z0-9 _-]+$")
        );
    }

    #[test]
    fn test_pattern_preserves_space_before_underscore() {
        assert_eq!(
            pattern_of("- Pattern: ^[a-z _]+$").as_deref(),
            Some("^[a-z _]+$")
        );
    }

    #[test]
    fn test_pattern_preserves_single_underscores() {
        assert_eq!(
            pattern_of("- Pattern: ^a _b_ c$").as_deref(),
            Some("^a _b_ c$")
        );
    }

    #[test]
    fn test_pattern_preserves_single_asterisks() {
        assert_eq!(
            pattern_of("- Pattern: ^a *b* c$").as_deref(),
            Some("^a *b* c$")
        );
    }

    #[test]
    fn test_pattern_preserves_literal_double_asterisk() {
        assert_eq!(
            pattern_of("- Pattern: ^a**b**c$").as_deref(),
            Some("^a**b**c$")
        );
    }

    #[test]
    fn test_pattern_preserves_double_underscore() {
        assert_eq!(
            pattern_of("- Pattern: ^a__b__c$").as_deref(),
            Some("^a__b__c$")
        );
    }

    #[test]
    fn test_pattern_preserves_square_brackets() {
        assert_eq!(pattern_of("- Pattern: ^a [b]$").as_deref(), Some("^a [b]$"));
        assert_eq!(
            pattern_of("- Pattern: ^[A-Z] [a-z]$").as_deref(),
            Some("^[A-Z] [a-z]$")
        );
    }

    #[test]
    fn test_pattern_preserves_backslashes() {
        assert_eq!(
            pattern_of(r"- Pattern: ^a\d+ b$").as_deref(),
            Some(r"^a\d+ b$")
        );
        assert_eq!(
            pattern_of(r"- Pattern: ^\d{2} \d{2}$").as_deref(),
            Some(r"^\d{2} \d{2}$")
        );
    }

    #[test]
    fn test_pattern_preserves_alternation_pipe() {
        assert_eq!(
            pattern_of("- Pattern: ^(a|b) c$").as_deref(),
            Some("^(a|b) c$")
        );
    }

    #[test]
    fn test_pattern_preserves_unicode() {
        assert_eq!(
            pattern_of("- Pattern: ^[äöü ß]+$").as_deref(),
            Some("^[äöü ß]+$")
        );
    }

    #[test]
    fn test_pattern_preserves_consecutive_spaces() {
        assert_eq!(
            pattern_of("- Pattern: ^a  b$").as_deref(),
            Some("^a  b$"),
            "spacing within a line must be preserved verbatim"
        );
    }

    #[test]
    fn test_regex_key_behaves_like_pattern() {
        assert_eq!(
            pattern_of("- Regex: ^[A-Z] [a-z]$").as_deref(),
            Some("^[A-Z] [a-z]$")
        );
    }

    //
    // Descriptions are prose and are joined across continuation lines. This
    // behaviour is locked by a large number of golden files and must not change.
    //

    #[test]
    fn test_description_soft_line_break_is_joined_with_single_space() {
        let content = "---\nprefix: \"t\"\nrepo: \"https://e.org/\"\n---\n\n### P\n\n- a\n  - Type: string\n  - Description: The name of the test. This is a unique identifier\n    that helps track individual test cases across the system.\n    It should be descriptive.\n";
        let model = parse_markdown(content, None).expect("model should parse");

        assert_eq!(
            model.objects[0].attributes[0].docstring,
            "The name of the test. This is a unique identifier that helps track \
             individual test cases across the system. It should be descriptive."
        );
    }

    #[test]
    fn test_description_preserves_inline_markup() {
        let model = parse_with_option("- Description: a *b* and _c_ here");
        assert_eq!(
            model.objects[0].attributes[0].docstring,
            "a *b* and _c_ here"
        );
    }

    //
    // Array markers and types must keep working now that the `[`/`]` inline
    // special-cases have been removed in favour of reading the source directly.
    //

    #[test]
    fn test_array_type_is_still_detected() {
        let model = parse_with_option("- Term: schema:x");
        assert!(!model.objects[0].attributes[0].is_array);

        let content = "---\nprefix: \"t\"\nrepo: \"https://e.org/\"\n---\n\n### P\n\n- a\n  - Type: string[]\n";
        let model = parse_markdown(content, None).expect("model should parse");
        assert!(
            model.objects[0].attributes[0].is_array,
            "`string[]` must still be recognised as an array"
        );
        assert_eq!(model.objects[0].attributes[0].dtypes, vec!["string"]);
    }

    #[test]
    fn test_object_reference_link_is_stripped_to_type_name() {
        let content = "---\nprefix: \"t\"\nrepo: \"https://e.org/\"\n---\n\n### P\n\n- a\n  - Type: [Other](#other)[]\n\n### Other\n\n- b\n  - Type: string\n";
        let model = parse_markdown(content, None).expect("model should parse");

        assert_eq!(model.objects[0].attributes[0].dtypes, vec!["Other"]);
        assert!(model.objects[0].attributes[0].is_array);
    }

    #[test]
    fn test_tab_indented_options_still_parse() {
        let content = "---\nprefix: \"t\"\nrepo: \"https://e.org/\"\n---\n\n### P\n\n- a\n\t- Type: string\n\t- Pattern: ^[a-z ]+$\n";
        let model = parse_markdown(content, None).expect("model should parse");

        assert_eq!(model.objects[0].attributes[0].dtypes, vec!["string"]);
        assert!(model.objects[0].attributes[0]
            .options
            .iter()
            .any(|o| matches!(o, AttrOption::Pattern(p) if p == "^[a-z ]+$")));
    }

    //
    // `__name__` and `**name**` both mean "required". This is load-bearing for
    // most fixtures in the corpus.
    //

    #[test]
    fn test_bold_attribute_names_are_required() {
        for marker in ["**name**", "__name__"] {
            let content = format!(
                "---\nprefix: \"t\"\nrepo: \"https://e.org/\"\n---\n\n### P\n\n- {marker}\n  - Type: string\n"
            );
            let model = parse_markdown(&content, None).expect("model should parse");

            assert_eq!(model.objects[0].attributes[0].name, "name");
            assert!(
                model.objects[0].attributes[0].required,
                "{marker} must mark the attribute as required"
            );
        }
    }

    //
    // Only a level-3 heading declares an object. Objects written at any other
    // level is reported: a level-2 heading drops its list, and a level-4
    // heading would otherwise append its attributes to the previous object.
    //

    /// Parses a fixture and returns the validation errors it produces.
    fn errors_for(path: &str) -> Vec<crate::validation::ValidationError> {
        let content = std::fs::read_to_string(path).expect("Could not read fixture");
        match parse_markdown(&content, None) {
            Ok(_) => Vec::new(),
            Err(validator) => validator.errors,
        }
    }

    #[test]
    fn test_object_at_h4_is_reported() {
        let errors = errors_for("tests/data/model_wrong_heading_level.md");
        let error = errors
            .iter()
            .find(|e| e.object.as_deref() == Some("Dropped"))
            .unwrap_or_else(|| unreachable!("no diagnostic for the level-4 object: {errors:#?}"));

        assert_eq!(error.error_type, ErrorType::ObjectError);
        assert!(error.message.contains("level-4"), "{}", error.message);
        assert_eq!(error.positions[0].line, 11);
    }

    #[test]
    fn test_object_at_h2_is_reported() {
        let errors = errors_for("tests/data/model_wrong_heading_level.md");
        let error = errors
            .iter()
            .find(|e| e.object.as_deref() == Some("AlsoDropped"))
            .unwrap_or_else(|| unreachable!("no diagnostic for the level-2 object: {errors:#?}"));

        assert_eq!(error.error_type, ErrorType::ObjectError);
        assert!(error.message.contains("level-2"), "{}", error.message);
        assert_eq!(error.positions[0].line, 16);
    }

    /// The diagnostic must name the fix.
    #[test]
    fn test_misplaced_object_diagnostic_suggests_h3() {
        let errors = errors_for("tests/data/model_wrong_heading_level.md");
        let error = errors
            .iter()
            .find(|e| e.object.as_deref() == Some("Dropped"))
            .expect("no diagnostic");

        let solution = error.solution.clone().expect("no solution offered");
        assert!(solution.contains("### Dropped"), "{solution}");
    }

    /// Attributes under a level-4 heading must not join the previous object.
    #[test]
    fn test_h4_attributes_do_not_leak_into_previous_object() {
        let content =
            std::fs::read_to_string("tests/data/model_wrong_heading_level.md").expect("read");
        // The model is invalid, so inspect the objects the parser built.
        let mut model = DataModel::new(None, None);
        let line_offsets = create_line_offsets(&content);
        let mut context = ParserContext::default();
        let (objects, _) =
            parse_model_components(&content, &line_offsets, &mut model, &mut context);

        let good = objects
            .iter()
            .find(|o| o.name == "Good")
            .expect("Good object missing");
        let names: Vec<&str> = good.attributes.iter().map(|a| a.name.as_str()).collect();

        assert_eq!(
            names,
            vec!["a"],
            "attributes from the level-4 heading leaked into the previous object"
        );
    }

    //
    // The diagnostic must not fire on legitimate documents.
    //

    #[test]
    fn test_narrative_headings_are_not_reported() {
        let errors = errors_for("tests/data/model_narrative_headings.md");
        assert!(
            errors.is_empty(),
            "narrative headings and enumerations must not be flagged: {errors:#?}"
        );
    }

    #[test]
    fn test_enumeration_section_is_not_reported() {
        let errors = errors_for("tests/data/model.md");
        assert!(
            errors.is_empty(),
            "the '## Enumerations' section must not be flagged: {errors:#?}"
        );
    }

    /// A level-1 heading names the model, so its preamble is not an object.
    #[test]
    fn test_h1_title_preamble_is_not_reported() {
        // This fixture declares a relative import, so it must be parsed with
        // its path rather than from a bare string.
        let result = DataModel::from_markdown(std::path::Path::new("tests/data/model_meta.md"));
        assert!(
            result.is_ok(),
            "a level-1 title must not be reported as a misplaced object: {:#?}",
            result.err()
        );
    }

    #[test]
    fn test_full_documentation_model_is_not_reported() {
        let errors = errors_for("tests/data/model_full_documentation.md");
        assert!(
            errors.is_empty(),
            "documentation headings must not be flagged: {errors:#?}"
        );
    }

    //
    // Continuation lines. CommonMark only opens a list when the marker is
    // followed by whitespace, so `-10`, `+1` and `*emphasis*` are lazy
    // paragraph continuations and belong to the value.
    //

    #[test]
    fn test_continuation_line_starting_with_hyphen_is_kept() {
        let content = "---\nprefix: \"t\"\nrepo: \"https://e.org/\"\n---\n\n### P\n\n- a\n  - Type: string\n  - Description: values from\n    -10 upward\n";
        let model = parse_markdown(content, None).expect("model should parse");

        assert_eq!(
            model.objects[0].attributes[0].docstring,
            "values from -10 upward"
        );
    }

    #[test]
    fn test_continuation_line_starting_with_plus_is_kept() {
        let content = "---\nprefix: \"t\"\nrepo: \"https://e.org/\"\n---\n\n### P\n\n- a\n  - Type: string\n  - Description: charge of\n    +1 for a cation\n";
        let model = parse_markdown(content, None).expect("model should parse");

        assert_eq!(
            model.objects[0].attributes[0].docstring,
            "charge of +1 for a cation"
        );
    }

    #[test]
    fn test_wrapped_pattern_is_not_truncated() {
        let content = "---\nprefix: \"t\"\nrepo: \"https://e.org/\"\n---\n\n### P\n\n- a\n  - Type: string\n  - Pattern: ^(abc|\n    -def)$\n";
        let model = parse_markdown(content, None).expect("model should parse");

        let pattern = model.objects[0].attributes[0]
            .options
            .iter()
            .find_map(|o| match o {
                AttrOption::Pattern(p) => Some(p.clone()),
                _ => None,
            })
            .expect("pattern option missing");

        assert!(
            pattern.contains("-def"),
            "the wrapped half of the regex was dropped: {pattern:?}"
        );
    }

    /// An item reported without a bullet marker must keep its text. Dropping it
    /// produced an empty attribute name, which the validator then indexed into.
    #[test]
    fn test_ordered_sublist_does_not_panic() {
        let content = "---\nprefix: \"t\"\nrepo: \"https://e.org/\"\n---\n\n### P\n\n- a\n  - Type: string\n  - Description: steps\n    1. one\n";
        let result = parse_markdown(content, None);

        // Whatever the verdict, it must be a diagnostic rather than a panic.
        if let Err(validator) = result {
            assert!(!validator.errors.is_empty());
        }
    }

    #[test]
    fn test_strip_list_marker_requires_whitespace() {
        assert_eq!(strip_list_marker("- item"), Some("item"));
        assert_eq!(strip_list_marker("*  item"), Some("item"));
        assert_eq!(strip_list_marker("+\titem"), Some("item"));
        assert_eq!(strip_list_marker("-10 upward"), None);
        assert_eq!(strip_list_marker("+1 charge"), None);
        assert_eq!(strip_list_marker("*emphasis*"), None);
        assert_eq!(strip_list_marker("plain"), None);
    }

    //
    // The misplaced-object diagnostic must not fire on prose. Turning a valid
    // document invalid is worse than missing a misplaced object, and a page
    // documenting the dialect's own option keys is a realistic document.
    //

    /// Builds a model with one valid object followed by `trailer`.
    #[allow(clippy::result_large_err)]
    fn parse_with_trailer(trailer: &str) -> Result<DataModel, Validator> {
        let content = format!(
            "---\nprefix: \"t\"\nrepo: \"https://e.org/\"\n---\n\n### Good\n\n- a\n  - Type: string\n\n{trailer}\n"
        );
        parse_markdown(&content, None)
    }

    #[test]
    fn test_narrative_lists_do_not_trigger_diagnostic() {
        let cases = [
            "## Glossary\n\n- Term: a word used in this document\n- Type: a category of thing",
            "## Attribute options\n\n- Type: the data type\n- Description: free text",
            "## Design notes\n\n- Description: fields should be short.",
            "## Notes\n\n- outer bullet\n  - Type: mentioned in a nested narrative bullet",
            "## Overview\n\n- Description: this model covers experiments.",
        ];

        for case in cases {
            assert!(
                parse_with_trailer(case).is_ok(),
                "narrative prose must not be reported as a misplaced object:\n{case}"
            );
        }
    }

    #[test]
    fn test_fenced_example_does_not_trigger_diagnostic() {
        let trailer = "## How to write a model\n\n- Example:\n  ```markdown\n  - name\n    - Type: string\n  ```";
        assert!(
            parse_with_trailer(trailer).is_ok(),
            "a fenced example is documentation, not a declaration"
        );
    }

    #[test]
    fn test_shorthand_object_at_wrong_level_is_reported() {
        let result = parse_with_trailer("#### Dropped\n\n- b: string\n- c: integer");
        assert!(
            result.is_err(),
            "shorthand attributes under a level-4 heading must be reported"
        );
    }

    /// A narrative list before the real attribute list must not consume the check.
    #[test]
    fn test_diagnostic_survives_a_leading_narrative_list() {
        let result =
            parse_with_trailer("#### Dropped\n\n- narrative\n\nProse.\n\n- b\n  - Type: string");
        assert!(
            result.is_err(),
            "the attribute list further down must still be reported"
        );
    }

    //
    // Malformed input must produce a diagnostic, never a panic, since
    // `parse_markdown` returns `Result<DataModel, Validator>`.
    //

    /// Every one of these aborted the process before. The assertion is simply
    /// that the call *returns* — a panic fails the test by unwinding.
    #[test]
    fn test_malformed_input_never_panics() {
        let cases = [
            (
                "single asterisk name",
                "### O\n\n- *name*\n  - Type: string\n",
            ),
            (
                "triple asterisk name",
                "### O\n\n- ***name***\n  - Type: string\n",
            ),
            ("empty bold name", "### O\n\n- ****\n  - Type: string\n"),
            ("bare list item", "### O\n\n- \n  - Type: string\n"),
            ("code span name", "### O\n\n- `name`\n  - Type: string\n"),
            (
                "code span heading",
                "### `Weird`\n\n- a\n  - Type: string\n",
            ),
            ("unclosed mixin", "### A [\n\n- name\n  - Type: string\n"),
            ("empty mixin", "### A []\n\n- name\n  - Type: string\n"),
            (
                "unknown mixin",
                "### A [NoSuchMixin]\n\n- name\n  - Type: string\n",
            ),
            (
                "unparseable option",
                "### O\n\n- a\n  - Type: string\n  - Minimum: abc\n",
            ),
            (
                "array plus union",
                "### O\n\n- a\n  - Type: Other, Another[]\n",
            ),
            ("orphan type annotation", ": orphan\n\ntext\n"),
            ("empty heading", "###\n\n- a\n  - Type: string\n"),
            (
                "ordered sub-list",
                "### O\n\n- a\n  - Type: string\n  - Description: s\n    1. one\n",
            ),
            ("nothing at all", ""),
            ("only frontmatter", "---\nprefix: t\n---\n"),
            ("attributes before any object", "- a\n  - Type: string\n"),
        ];

        for (label, content) in cases {
            // Either verdict is acceptable; a panic fails the test.
            let _ = parse_markdown(content, None);
            let _ = label;
        }
    }

    /// The diagnostics must be usable: named error type, and a line number
    /// pointing into the model rather than into the parser.
    #[test]
    fn test_malformed_input_reports_a_located_diagnostic() {
        let cases: [(&str, ErrorType, usize); 3] = [
            (
                "### O\n\n- *name*\n  - Type: string\n",
                ErrorType::NameError,
                3,
            ),
            (
                "### A [\n\n- name\n  - Type: string\n",
                ErrorType::ObjectError,
                1,
            ),
            (
                "### O\n\n- a\n  - Type: string\n  - Minimum: abc\n",
                ErrorType::TypeError,
                3,
            ),
        ];

        for (content, expected_type, expected_line) in cases {
            let validator =
                parse_markdown(content, None).expect_err("malformed input must not validate");

            let error = validator
                .errors
                .iter()
                .find(|e| e.error_type == expected_type)
                .unwrap_or_else(|| {
                    unreachable!(
                        "expected {expected_type} for {content:?}: {:#?}",
                        validator.errors
                    )
                });

            assert!(error.solution.is_some(), "a diagnostic must suggest a fix");
            assert_eq!(
                error.positions.first().map(|p| p.line),
                Some(expected_line),
                "wrong line for {content:?}"
            );
        }
    }

    /// An unreadable import is a model error, not a reason to abort.
    #[test]
    fn test_unreadable_import_is_a_diagnostic() {
        let content = "---\nimports:\n  other: ./definitely-not-here.md\n---\n\n### O\n\n- a\n  - Type: string\n";
        let validator =
            parse_markdown(content, None).expect_err("a missing import must be reported");

        assert!(
            validator
                .errors
                .iter()
                .any(|e| e.message.contains("import")),
            "expected an import diagnostic: {:#?}",
            validator.errors
        );
    }

    /// An empty or multi-byte name must not be indexed into.
    #[test]
    fn test_empty_and_unicode_names_do_not_panic() {
        for content in [
            "### O\n\n- \u{00e4}bc\n  - Type: string\n",
            "### \u{00e4}O\n\n- a\n  - Type: string\n",
            "### 1O\n\n- a\n  - Type: string\n",
        ] {
            let _ = parse_markdown(content, None);
        }
    }
}
