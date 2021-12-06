cd ./services/rest_frontent/ || exit
docker build -t niel4265/rest-exchange:latest .
docker push niel4265/rest-exchange:latest

cd ../price || exit
docker build -t niel4265/price-exchange:latest .
docker push niel4265/price-exchange:latest

cd ../owner || exit
docker build -t niel4265/owner-exchange:latest .
docker push niel4265/owner-exchange:latest

cd ../../orderbook/ || exit

docker build -t niel4265/orderbook:latest .
docker push niel4265/orderbook:latest
