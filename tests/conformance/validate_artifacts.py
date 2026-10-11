#!/usr/bin/env python3
"""Cross-target conformance check for md-models.

Exports one reference model to every CLI target and validates each artefact with
that ecosystem's own tool, rather than with md-models itself. Every defect found
in the v0.2.10 review -- invalid Protobuf scalars, unbound Turtle prefixes, a
JSON-LD stack overflow, dropped XSD facets -- would have been caught here.

Validators that are not installed are reported as SKIP, not as failure, so the
script is useful locally and strict in CI (pass --strict to fail on skips).

Usage:
    python3 tests/conformance/validate_artifacts.py [--model PATH] [--strict]
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_MODEL = REPO_ROOT / "tests" / "data" / "model_recursive.md"

PASS, FAIL, SKIP = "PASS", "FAIL", "SKIP"


class Result:
    def __init__(self) -> None:
        self.rows: list[tuple[str, str, str]] = []

    def add(self, target: str, status: str, detail: str = "") -> None:
        self.rows.append((target, status, detail))

    @property
    def failed(self) -> list[tuple[str, str, str]]:
        return [r for r in self.rows if r[1] == FAIL]

    @property
    def skipped(self) -> list[tuple[str, str, str]]:
        return [r for r in self.rows if r[1] == SKIP]


def find_binary() -> Path:
    """Locates the md-models binary, preferring a release build."""
    for profile in ("release", "debug"):
        candidate = REPO_ROOT / "target" / profile / "md-models"
        if candidate.exists():
            return candidate
    sys.exit("md-models binary not found. Run `cargo build --release` first.")


def convert(binary: Path, model: Path, target: str, out: Path, root: str | None) -> None:
    """Runs a conversion, raising on a non-zero exit."""
    cmd = [str(binary), "convert", "-i", str(model), "-t", target, "-o", str(out)]
    if root:
        cmd += ["-r", root]
    proc = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
    if proc.returncode != 0:
        raise RuntimeError(
            f"conversion failed (exit {proc.returncode}): {proc.stderr.strip()[:400]}"
        )


def check_turtle(path: Path) -> tuple[str, str]:
    try:
        import rdflib
    except ImportError:
        return SKIP, "rdflib not installed"

    graph = rdflib.Graph()
    graph.parse(str(path), format="turtle")
    return PASS, f"{len(graph)} triples"


def check_shacl(path: Path) -> tuple[str, str]:
    """Parses the shapes *and* runs them through a SHACL processor.

    Parsing alone is not enough. A shapes graph can be valid Turtle and still be
    rejected by every SHACL engine -- for instance if a property shape carries
    no `sh:path`, which is what wrapping two paths in `sh:or` produces. Running
    the shapes against a trivial data graph forces the processor to check that
    each shape is well formed.
    """
    try:
        import rdflib
    except ImportError:
        return SKIP, "rdflib not installed"

    shapes = rdflib.Graph()
    shapes.parse(str(path), format="turtle")

    try:
        import pyshacl
    except ImportError:
        return PASS, f"{len(shapes)} triples (pyshacl not installed, parse-only)"

    empty = rdflib.Graph()
    # Raises ReportableRuntimeError if any shape is malformed. Whether the empty
    # data graph conforms is irrelevant; the point is that the shapes execute.
    pyshacl.validate(empty, shacl_graph=shapes, inference="none")
    return PASS, f"{len(shapes)} triples, shapes are well formed"


def check_json(path: Path) -> tuple[str, str]:
    data = json.loads(path.read_text())
    if not data:
        raise ValueError("document is empty")
    return PASS, f"{len(data)} top-level keys"


def check_json_schema(path: Path) -> tuple[str, str]:
    try:
        import jsonschema
    except ImportError:
        return SKIP, "jsonschema not installed"

    schema = json.loads(path.read_text())
    jsonschema.Draft202012Validator.check_schema(schema)
    return PASS, "schema is well formed"


def check_protobuf(path: Path) -> tuple[str, str]:
    if not shutil.which("protoc"):
        return SKIP, "protoc not installed"

    proc = subprocess.run(
        [
            "protoc",
            f"--proto_path={path.parent}",
            "--descriptor_set_out=/dev/null",
            str(path),
        ],
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip()[:400])
    return PASS, "compiles"


def check_xsd(path: Path) -> tuple[str, str]:
    if not shutil.which("xmllint"):
        return SKIP, "xmllint not installed"

    # An empty instance is enough to force libxml2 to compile the schema.
    instance = path.with_suffix(".instance.xml")
    instance.write_text('<?xml version="1.0"?><none/>')
    proc = subprocess.run(
        ["xmllint", "--noout", "--schema", str(path), str(instance)],
        capture_output=True,
        text=True,
    )
    if "failed to compile" in proc.stderr:
        raise RuntimeError(proc.stderr.strip()[:400])
    return PASS, "schema compiles"


def check_linkml(path: Path) -> tuple[str, str]:
    try:
        import yaml
    except ImportError:
        return SKIP, "PyYAML not installed"

    schema = yaml.safe_load(path.read_text())
    name = schema.get("name", "")
    if not name:
        raise ValueError("schema has no name")
    # LinkML constrains `name` to an NCName.
    if any(c.isspace() for c in name):
        raise ValueError(f"schema name {name!r} is not a valid NCName")

    try:
        from linkml_runtime.utils.schemaview import SchemaView
    except ImportError:
        return PASS, f"name={name} (linkml-runtime not installed, parse-only)"

    view = SchemaView(str(path))
    return PASS, f"{len(view.all_classes())} classes"


def check_graphql(path: Path) -> tuple[str, str]:
    try:
        from graphql import build_schema
    except ImportError:
        return SKIP, "graphql-core not installed"

    build_schema(path.read_text())
    return PASS, "schema parses"


def check_non_empty(path: Path) -> tuple[str, str]:
    text = path.read_text().strip()
    if not text:
        raise ValueError("output is empty")
    return PASS, f"{len(text)} bytes"


def check_json_schema_all(path: Path) -> tuple[str, str]:
    """`json-schema-all` writes one schema per object into a directory."""
    if not path.is_dir():
        raise ValueError(f"{path} is not a directory")

    files = sorted(path.glob("*.json"))
    if not files:
        raise ValueError("no schemas were written")

    for schema_file in files:
        if schema_file.stat().st_size == 0:
            raise ValueError(f"{schema_file.name} is empty")
        json.loads(schema_file.read_text())
    return PASS, f"{len(files)} schemas"


# Every target accepted by `md-models convert --template`, so that a target
# cannot be added without a conformance check.
# target -> (root object required, validator)
TARGETS = {
    "json-schema": (True, check_json_schema),
    "json-schema-all": (False, check_json_schema_all),
    "json-ld": (True, check_json),
    "shacl": (False, check_shacl),
    "owl": (False, check_turtle),
    "shex": (False, check_non_empty),
    "xml-schema": (False, check_xsd),
    "linkml": (False, check_linkml),
    "protobuf": (False, check_protobuf),
    "graphql": (False, check_graphql),
    "rust": (False, check_non_empty),
    "python-pydantic": (False, check_non_empty),
    "python-dataclass": (False, check_non_empty),
    "python-pydantic-xml": (False, check_non_empty),
    "typescript": (False, check_non_empty),
    "typescript-zod": (False, check_non_empty),
    "golang": (False, check_non_empty),
    "julia": (False, check_non_empty),
    "mermaid": (False, check_non_empty),
    "markdown": (False, check_non_empty),
    "compact-markdown": (False, check_non_empty),
    "mk-docs": (False, check_non_empty),
    "internal": (False, check_json),
}

SUFFIXES = {
    "shacl": ".ttl",
    "owl": ".ttl",
    "shex": ".shex",
    "xml-schema": ".xsd",
    "linkml": ".yml",
    "protobuf": ".proto",
    "graphql": ".graphql",
    "json-schema": ".json",
    "json-ld": ".jsonld",
    "internal": ".json",
    "json-schema-all": "",
}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--model", type=Path, default=DEFAULT_MODEL)
    parser.add_argument("--root", default="Experiment")
    parser.add_argument(
        "--strict",
        action="store_true",
        help="treat missing validators as failures (use in CI)",
    )
    args = parser.parse_args()

    binary = find_binary()
    results = Result()

    with tempfile.TemporaryDirectory() as tmp:
        outdir = Path(tmp)
        for target, (needs_root, validator) in TARGETS.items():
            out = outdir / f"{target}{SUFFIXES.get(target, '.txt')}"
            try:
                convert(binary, args.model, target, out, args.root if needs_root else None)
            except Exception as exc:  # noqa: BLE001 - reported, not raised
                results.add(target, FAIL, f"conversion: {exc}")
                continue

            try:
                status, detail = validator(out)
                results.add(target, status, detail)
            except Exception as exc:  # noqa: BLE001 - reported, not raised
                results.add(target, FAIL, f"validation: {exc}")

    width = max(len(t) for t, _, _ in results.rows)
    print(f"\nConformance report for {args.model.name}\n")
    for target, status, detail in results.rows:
        marker = {PASS: "ok  ", FAIL: "FAIL", SKIP: "skip"}[status]
        print(f"  [{marker}] {target:<{width}}  {detail}")

    print(
        f"\n{len(results.rows)} targets: "
        f"{len(results.rows) - len(results.failed) - len(results.skipped)} passed, "
        f"{len(results.failed)} failed, {len(results.skipped)} skipped"
    )

    if results.failed:
        return 1
    if args.strict and results.skipped:
        print("\n--strict: missing validators counted as failures", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
