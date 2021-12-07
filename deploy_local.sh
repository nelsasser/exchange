kubectl apply -f ./secrets/local_db_secrets.yaml

kubectl apply -f ./owner/owner-deployment.yaml

kubectl apply -f ./price/price-deployment.yaml

kubectl apply -f ./rest_frontend/rest-deployment.yaml
kubectl apply -f ./rest_frontend/rest-service.yaml

kubectl port-forward --address 0.0.0.0 service/rest-service 5000:5000 &
