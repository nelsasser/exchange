import os

pth = 'C:\\Users\\elsan\\Documents\\exchange\\pubsub_keys.json'
if os.path.exists(pth):
    os.environ['GOOGLE_APPLICATION_CREDENTIALS'] = pth
