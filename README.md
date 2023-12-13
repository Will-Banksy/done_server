# done.

`done_server` is a todo list web application running on the Rocket web framework intended for deployment in the cloud, developed for my IoT & Cloud module at uni. The corresponding client for IoT (specifically raspberry pi) devices will be available under the name `done_rpi_client`.

## Deploying

See the [aws_deploy.sh](aws_deploy.sh) bash script for semi-automated AWS deploying.

See the [aws_dev_setup.sh](aws_dev_setup.sh) bash script for automatically setting up a database with AWS for dev work.

For TLS, either put `done_server` behind a reverse proxy that supports TLS such as [Caddy](https://caddyserver.com/) or manually configuring Rocket to use TLS (refer to the [docs](https://rocket.rs/v0.5/guide/configuration/#tls)).

## API Usage

`done_server` exposes a basic JSON REST API. To use the API, a user must be registered and credentials supplied via HTTP Basic Authentication, as shown below.

3 endpoints are exposed by the API: /api/tasks/get, /api/tasks/set and /api/tasks/delete for getting, setting and deleting tasks respectively.

### Get Tasks

```
curl -v -X POST -u <username>:<password> <host>/api/tasks/get
```

### Set Tasks

```
curl -v -X POST -u <username>:<password> <host>/api/tasks/set -H "Content-Type: application/json" -d '[{"user_task_id": 0, "task": "<task_description>"}, {"user_task_id": 1, "task": "<task_description>"}]'
```

### Delete Tasks

```
curl -v -X POST -u <username>:<password> <host>/api/tasks/delete -H "Content-Type: application/json" -d '[1]'
```