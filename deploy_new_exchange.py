import os

keypath = 'C:\\Users\\elsan\\Documents\\exchange\\admin_keys.json'
if os.path.exists(keypath):
    os.environ['GOOGLE_APPLICATION_CREDENTIALS'] = keypath

import sys
import json
import google.oauth2.service_account as service_account
import googleapiclient.discovery
from google.cloud import storage
import mysql.connector
from time import sleep

db_config = json.load(open('./db_config.json'))

def execute_sql(query, params=None, mode='select', db=None):
    if db not in ('price', 'owner'):
        raise ValueError('db must be one of `price` or `owner`')
    with mysql.connector.connect(**db_config, database=db) as conn:
        cursor = conn.cursor()
        cursor.execute(query, params)
        if mode == 'select':
            return cursor.fetchall()
        elif mode == 'commit':
            conn.commit()


def setup_bucket(name):
    bucket_name = f'{name.lower()}-events-bucket'
    # create bucket for raw event storage
    storage.Client().create_bucket(bucket_name)

    return bucket_name


def setup_topics(topic_name, creds):
    # generate new topics and subscriptions to the topics
    pubsub_service = googleapiclient.discovery.build('pubsub', 'v1', credentials=creds)

    # pubsub input to matching engine
    book_topic = f'projects/project-steelieman/%s/{topic_name}'
    pubsub_service.projects().topics().create(name=book_topic % 'topics').execute()
    pubsub_service.projects().subscriptions().create(name=f'{book_topic}-sub' % 'subscriptions',
                                                     body={'topic': book_topic % 'topics'}).execute()

    # output of matching engine
    events_topic = f'{book_topic}-Events'
    pubsub_service.projects().topics().create(name=events_topic % 'topics').execute()
    pubsub_service.projects().subscriptions().create(name=f'{events_topic}-price-sub' % 'subscriptions',
                                                     body={'topic': events_topic % 'topics'}).execute()
    pubsub_service.projects().subscriptions().create(name=f'{events_topic}-account-sub' % 'subscriptions',
                                                     body={'topic': events_topic % 'topics'}).execute()

    return book_topic, events_topic


def events_bucket_route(name, events_topic, bucket_name, creds):
    dataflow_service = googleapiclient.discovery.build('dataflow', 'v1b3', credentials=creds)
    gcs_path = "gs://dataflow-templates-us-central1/latest/Cloud_PubSub_to_GCS_Text"
    job_data = {
        "jobName": f"ps-to-text-{name}-Events",
        "environment": {
            "bypassTempDirValidation": False,
            "tempLocation": f"gs://{bucket_name}/temp",
            "ipConfiguration": "WORKER_IP_UNSPECIFIED",
            "additionalExperiments": []
        },
        "parameters": {
            "inputTopic": events_topic % 'topics',
            "outputDirectory": f"gs://{bucket_name}",
            "outputFilenamePrefix": f"{name.lower()}-events-"
        }
    }

    dataflow_service.projects().templates().launch(projectId='project-steelieman',
                                                   gcsPath=gcs_path,
                                                   body=job_data).execute() # job to automatically pipe output from matches to a log


def events_price_route(name, events_topic, bucket_name, creds):
    dataflow_service = googleapiclient.discovery.build('dataflow', 'v1b3', credentials=creds)
    gcs_path = "gs://dataflow-templates-us-central1/latest/Cloud_PubSub_to_Cloud_PubSub"
    job_data = {
        "jobName": f"{name}-Events-to-price-updates",
        "environment": {
            "bypassTempDirValidation": False,
            "tempLocation": f"gs://{bucket_name}/price-temp",
            "ipConfiguration": "WORKER_IP_UNSPECIFIED",
            "additionalExperiments": []
        },
        "parameters": {
            "inputSubscription": f"{events_topic % 'subscriptions'}-price-sub",
            "outputTopic": "projects/project-steelieman/topics/price-updates"
        }
    }

    dataflow_service.projects().templates().launch(projectId='project-steelieman',
                                                   gcsPath=gcs_path,
                                                   body=job_data).execute() # job to automatically pipe output from matches to a log


def events_account_route(name, events_topic, bucket_name, creds):
    dataflow_service = googleapiclient.discovery.build('dataflow', 'v1b3', credentials=creds)
    gcs_path = "gs://dataflow-templates-us-central1/latest/Cloud_PubSub_to_Cloud_PubSub"
    job_data = {
        "jobName": f"{name}-Events-to-account-updates",
        "environment": {
            "bypassTempDirValidation": False,
            "tempLocation": f"gs://{bucket_name}/account-temp",
            "ipConfiguration": "WORKER_IP_UNSPECIFIED",
            "additionalExperiments": []
        },
        "parameters": {
            "inputSubscription": f"{events_topic % 'subscriptions'}-account-sub",
            "outputTopic": "projects/project-steelieman/topics/account-updates"
        }
    }

    dataflow_service.projects().templates().launch(projectId='project-steelieman',
                                                   gcsPath=gcs_path,
                                                   body=job_data).execute() # job to automatically pipe output from matches to a log


def setup_orderbook(name, creds):
    vm_service = googleapiclient.discovery.build('compute', 'v1', credentials=creds)

    data = {
        "canIpForward": False,
        "confidentialInstanceConfig": {
            "enableConfidentialCompute": False
        },
        "deletionProtection": False,
        "description": "",
        "disks": [
            {
                "autoDelete": True,
                "boot": True,
                "deviceName": "aapl-orderbook",
                "diskEncryptionKey": {},
                "initializeParams": {
                    "diskSizeGb": "10",
                    "diskType": "projects/project-steelieman/zones/us-central1-c/diskTypes/pd-balanced",
                    "labels": {},
                    "sourceImage": "projects/project-steelieman/global/images/orderbook"
                },
                "mode": "READ_WRITE",
                "type": "PERSISTENT"
            }
        ],
        "displayDevice": {
            "enableDisplay": False
        },
        "guestAccelerators": [],
        "labels": {},
        "machineType": "projects/project-steelieman/zones/us-central1-c/machineTypes/e2-medium",
        "metadata": {
            "items": [
                {
                    "key": "startup-script",
                    "value": f'#! /bin/bash\ncd ../../../../../srv\n./orderbook {name} >> logs.txt'
                }
            ]
        },
        "name": f"{name.lower()}-orderbook",
        "networkInterfaces": [
            {
                "accessConfigs": [
                    {
                        "name": "External NAT",
                        "networkTier": "PREMIUM"
                    }
                ],
                "subnetwork": "projects/project-steelieman/regions/us-central1/subnetworks/default"
            }
        ],
        "reservationAffinity": {
            "consumeReservationType": "ANY_RESERVATION"
        },
        "scheduling": {
            "automaticRestart": True,
            "onHostMaintenance": "MIGRATE",
            "preemptible": False
        },
        "serviceAccounts": [
            {
                "email": "129836438765-compute@developer.gserviceaccount.com",
                "scopes": [
                    "https://www.googleapis.com/auth/cloud-platform"
                ]
            }
        ],
        "shieldedInstanceConfig": {
            "enableIntegrityMonitoring": True,
            "enableSecureBoot": False,
            "enableVtpm": True
        },
        "tags": {
            "items": [
                "http-server",
                "https-server"
            ]
        },
        "zone": "projects/project-steelieman/zones/us-central1-c"
    }

    vm_service.instances().insert(project='project-steelieman', zone='us-central1-c', body=data).execute()


def create_price_table(name):
    insert_query = """
        CREATE table {}_price
        (
            market_time int   not null,
            low         float not null,
            high        float not null,
            open        float not null,
            close       float not null,
            volume      int   not null,
            constraint {}_price_pk
                primary key (market_time)
        );
    """.format(name.lower(), name.lower())

    execute_sql(insert_query, mode='commit', db='price')


if __name__ == '__main__':
    asset = sys.argv[1]

    credentials = service_account.Credentials.from_service_account_file(filename=keypath)

    # print('Setting up topics')
    # book_topic, events_topic = setup_topics(asset, credentials)
    # print('Set up topics:', ', '.join([book_topic, events_topic]))
    #
    # print('Setting up bucket')
    # bucket_name = setup_bucket(asset)
    # print('Setup bucket:', bucket_name)
    #
    # print('Setting up event routes')
    # events_bucket_route(asset, events_topic, bucket_name, credentials)
    # events_price_route(asset, events_topic, bucket_name, credentials)
    # events_account_route(asset, events_topic, bucket_name, credentials)
    # print('Done setting up event routes')

    print('Setting up orderbook vm.')
    setup_orderbook(asset, credentials)
    print('Done setting up orderbook vm.')

    print('Creating new asset price table')
    create_price_table(asset)
    print('Done creating price table.')

    print('All done setting up new asset exchange route.')

