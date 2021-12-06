import os

pth = 'C:\\Users\\elsan\\Documents\\exchange\\pubsub_keys.json'
if os.path.exists(pth):
    os.environ['GOOGLE_APPLICATION_CREDENTIALS'] = pth

from google.cloud import pubsub_v1

if __name__ == '__main__':
    topic = f'projects/{os.getenv("GOOGLE_CLOUD_PROJECT") or "project-steelieman"}/topics/{asset}-Events'
    subsciption = f'projects/{os.getenv("GOOGLE_CLOUD_PROJECT") or "project-steelieman"}/subscriptions/{asset}-Events-sub'

    with pubsub_v1.SubscriberClient() as subscriber:




