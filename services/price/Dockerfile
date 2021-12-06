FROM python:3.8-slim
LABEL MAINTAINER="nick elsasser <nelsasser@colorado.edu>"

COPY ./requirements.txt ./requirements.txt

RUN pip3 install -r requirements.txt

WORKDIR /srv
COPY main.py /srv

EXPOSE 5000

CMD ["python3", "main.py"]