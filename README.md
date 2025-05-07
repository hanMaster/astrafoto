# Bot and worker in docker

Make new image for bot:

cd bot
docker build -t hanmaster/bot:latest .
docker push hanmaster/bot:latest


Make new image for worker:

cd worker
docker build -t hanmaster/worker:latest .
docker push hanmaster/worker:latest