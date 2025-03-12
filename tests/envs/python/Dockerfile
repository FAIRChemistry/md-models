FROM python:3.11-alpine

WORKDIR /app

COPY tests/envs/python/ /app/

RUN pip install -r requirements.txt

CMD sh -c "python test_pydantic.py && python test_pydantic_xml.py && python test_dataclasses.py"
