import os

keypath = 'C:\\Users\\elsan\\Documents\\exchange\\admin_keys.json'
if os.path.exists(keypath):
    os.environ['GOOGLE_APPLICATION_CREDENTIALS'] = keypath

import sys
import google.oauth2.service_account as service_account
import googleapiclient.discovery
from google.cloud import storage

if __name__ == '__main__':
    name = sys.argv[1]

    credentials = service_account.Credentials.from_service_account_file(filename=keypath)
    project = 'project-steelieman'

    # generate new topics and subscriptions to the topics
    pubsub_service = googleapiclient.discovery.build('pubsub', 'v1', credentials=credentials)

    book_in = f'projects/{project}/%s/{name}'
    book_out = f'{book_in}-Events'

    pubsub_service.projects().topics().create(name=book_in % 'topics').execute() # input into matching engine
    pubsub_service.projects().topics().create(name=book_out % 'topics').execute() # raw matching output for raw storage & downstream services

    book_sub = f'{book_in}-sub'
    events_price_sub = f'{book_out}-price'
    events_owner_sub = f'{book_out}-owner'


    pubsub_service.projects().subscriptions().create(name=book_sub % 'subscriptions',
                                                     body={'topic': book_in % 'topics'}).execute() # input for orderbook
    pubsub_service.projects().subscriptions().create(name=events_price_sub % 'subscriptions',
                                                     body={'topic': book_out % 'topics'}).execute() # output of orderbook, input of pricing service
    pubsub_service.projects().subscriptions().create(name=events_price_sub % 'subscriptions',
                                                     body={'topic': book_out % 'topics'}).execute() # output of orderbook, input of owner account service

    # create bucket for raw event storage
    storage.Client().create_bucket(f'{name.lower()}-event-bucket')

    dataflow_service = googleapiclient.discovery.build('dataflow', 'v1b3', credentials=credentials)
    gcs_path = "gs://dataflow-templates-us-central1/latest/Cloud_PubSub_to_GCS_Text"
    job_data = {
        "jobName": f"ps-to-text-{name}-Events",
        "environment": {
            "bypassTempDirValidation": False,
            "tempLocation": f"gs://{name.lower()}-events-bucket/temp",
            "ipConfiguration": "WORKER_IP_UNSPECIFIED",
            "additionalExperiments": []
        },
        "parameters": {
            "inputTopic": f"projects/project-steelieman/topics/{name}-Events",
            "outputDirectory": f"gs://{name.lower()}-events-bucket",
            "outputFilenamePrefix": f"{name.lower()}-events-"
        }
    }

    dataflow_service.projects().templates().launch(projectId='project-steelieman',
                                                   gcsPath=gcs_path,
                                                   body=job_data).execute() # job to automatically pipe output from matches to a log

