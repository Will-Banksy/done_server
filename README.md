# done.

`done_server` will be a todo list web application running on the Rocket web framework intended for deployment in the cloud, developed for my IoT & Cloud module at uni. The corresponding client for IoT (specifically raspberry pi) devices will be available under the name `done_rpi_client`.

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