FROM python:3.11-alpine

WORKDIR /app

# Copy the test files from current directory

# Copy files from relative parent paths using proper context paths
COPY tests/envs/python/ /app/
COPY tests/data/expected_pydantic.py /app/gen_pydantic.py
COPY tests/data/expected_python_pydantic_xml.py /app/gen_pydantic_xml.py
COPY tests/data/expected_python_dc.py /app/gen_dataclasses.py

RUN pip install -r requirements.txt

CMD sh -c "python test_pydantic.py && python test_pydantic_xml.py && python test_dataclasses.py"
