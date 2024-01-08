# hastebin

## Installation
### Docker
Just pull the docker image:
´´´
docker pull ghcr.io/adridevelopsthings/hastebin:main
´´´
and run the docker image:
´´´
docker run --name hastebin -d -p 80:80 -v ./data:/data ghcr.io/adridevelopsthings/hastebin:main
´´´

### Build it yourself
Build a binary with cargo yourself:
```
cargo build --release
```

## Configuration
Configure your hastebin instance with environment variables:
- `LISTEN_ADDRESS`: default=`127.0.0.1:8000`, docker default=`0.0.0.0:80`
- `DATA_DIRECTORY`: path to the directory where the uploaded files should be stored, default=`data`, docker default=`/data`
- `MAX_FILE_SIZE`: maximum size a file can have, default=`1048576` (1 MB)
- `AUTO_DELETE_CHECK_INTERVAL`: in seconds, default=`120`
- `AUTO_DELETE_OLDER_THAN`: files older than this duration will be deleted (set this value to 0 to disable auto deletion), default=`172800`(2 days)
- `ID_LENGTH`: the length a generated file id should have, default=`10`
- `CHANGE_KEY_LENGTH`: the length a change key should have, default=`64`

## Endpoints

### Static endpoints
`/index.html` and `/*/*` will return a html file. `/index.js` will return a javascript file.

### Api endpoints
- *Getting the content of a file* Make a GET request to `/api/file/<file_id>/<file_name>` and you will get the file with a guessed mime type or a 404 error.
- *Creating a new file* Make a POST request to `/api/file/<file_name>` and put the content in the body and you will get json that looks like this: `{"id": "file_id", "change_key": "change_key"}` back. If your uploaded file is too big an error 400 with the body `File is too big` will be responded.
- *Modifying a file* Make a PUT request to `/api/file/<file_id>/<file_name>`, put the content in the body and set the `Change-Key` header to your change key. If the modification was successfull a status 204 will be responded. If your uploaded file is too big an error 400 with the body `File is too big` will be responded. If your change key is invalid an error 403 with the body `Invalid change key` will be responded.
- *Deleting a file* Make a DELETE request to `/api/file/<file_id>/<file_name>` and set the `Change-Key` header to your change key. If the deletion was successfull a status 204 will be responded. If your change key is invalid an error 403 with the body `Invalid change key` will be responded.
