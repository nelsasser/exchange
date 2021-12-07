kubectl apply -f ./services/secrets/local_db_secrets.yaml

kubectl apply -f ./services/owner/owner-deployment.yaml

kubectl apply -f ./services/price/price-deployment.yaml

kubectl apply -f ./services/rest_frontend/rest-deployment.yaml
kubectl apply -f ./services/rest_frontend/rest-service.yaml

kubectl port-forward --address 0.0.0.0 service/rest-exchange 5000:5000 &
