import os
import requests
import uuid

srv = os.getenv('REST') or '0.0.0.0:5000'

if __name__ == '__main__':
    r = requests.get(f'{srv}/')
    assert r.status_code == 200

    r = requests.get(f'{srv}/uuid')
    assert r.status_code == 200
    uuid.UUID(r.json()['uuid'])

    r = requests.post(f'{srv}/submit', json={
        'owner': str(uuid.uuid1()),
        'asset': 'AAPL',
        'direction': 'BID',
        'price': 10.0,
        'size': 100
    })
    assert r.status_code == 200

    r = requests.post(f'{srv}/cancel', json={
        'owner': str(uuid.uuid1()),
        'asset': 'AAPL',
        'order': str(uuid.uuid1()),
    })
    assert r.status_code == 200

    r = requests.post(f'{srv}/orders', json={
        'owner': str(uuid.uuid1()),
    })
    assert r.status_code == 200
