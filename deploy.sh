kubectl apply -f ./services/secrets/db_secrets.yaml

kubectl apply -f ./services/owner/owner-deployment.yaml

kubectl apply -f ./services/price/price-deployment.yaml

kubectl apply -f ./services/rest_frontend/rest-deployment.yaml
kubectl apply -f ./services/rest_frontend/rest-service.yaml
