import os
import requests
import uuid

srv = os.getenv('REST') or '192.168.0.207:5000'

if __name__ == '__main__':
    url = f'http://{srv}/'
    print('TEST:', url)
    r = requests.get(url)
    print(r.text)
    assert r.status_code == 200

    url = f'http://{srv}/uuid'
    print('TEST:', url)
    r = requests.get(url)
    print(r.json())
    assert r.status_code == 200
    uuid.UUID(r.json()['uuid'])

    url = f'http://{srv}/submit'
    print('TEST:', url)
    r = requests.post(url, json={
        'owner': str(uuid.uuid1()),
        'asset': 'AAPL',
        'direction': 'Bid',
        'price': 10.0,
        'size': 100
    })
    print(r.json())
    assert r.status_code == 200

    url = f'http://{srv}/submit'
    print('TEST:', url)
    r = requests.post(url, json={
        'owner': str(uuid.uuid1()),
        'asset': 'AAPL',
        'direction': 'Ask',
        'price': 10.0,
        'size': 100
    })
    print(r.json())
    assert r.status_code == 200

    url = f'http://{srv}/cancel'
    print('TEST:', url)
    r = requests.post(url, json={
        'owner': str(uuid.uuid1()),
        'asset': 'AAPL',
        'order': str(uuid.uuid1()),
    })
    print(r.json())
    assert r.status_code == 200

    url = f'http://{srv}/orders'
    print('TEST:', url)
    r = requests.post(url, json={
        'owner': str(uuid.uuid1()),
        'asset': 'AAPL',
    })
    print(r.json())
    assert r.status_code == 200
