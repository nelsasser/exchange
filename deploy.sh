kubectl apply -f ./secrets/db_secrets.yaml

kubectl apply -f ./owner/owner-deployment.yaml

kubectl apply -f ./price/price-deployment.yaml

kubectl apply -f ./rest_frontend/rest-deployment.yaml
kubectl apply -f ./rest_frontend/rest-service.yaml
