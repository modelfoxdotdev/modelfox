{
"title": "Run the App on Your Own Server."
}

If you do not want to use the cloud hosted tangram app at https://app.tangram.dev, you can run it yourself!

To get started, install the Tangram CLI and run `tangram app`. This runs the app in a configuration suitable for testing on a single computer. It stores data in the local filesystem and in a local SQLite database.

To run the app in production, you will need:

- A PostgreSQL database.
- An S3-compatible storage bucket.
- An SMTP server such as Amazon SES.
- A license. Send us an email at hello@tangram.dev to purchase one.

## Configuration

The tangram app is configured with a JSON file. You can pass a path to a configuration file with the `--config` flag. Otherwise, the app will look for a configuration file at `tangram/app.json` in your operating system's standard configuration directory:

| Platform | Location                             | Example                                  |
| -------- | ------------------------------------ | ---------------------------------------- |
| Linux    | `$XDG_CONFIG_DIR` or `$HOME/.config` | /home/alice/.config                      |
| macOS    | `$HOME/Library/Application Support`  | /Users/Alice/Library/Application Support |
| Windows  | `{FOLDERID_RoamingAppData}`          | C:\Users\Alice\AppData\Roaming           |

### auth

By default, the app has authentication disabled and anyone who can access the app can view and edit all data. If you enable authentication, users will have to enter a login code they receive via email. You must set the `smtp` field documented below for authentication emails to be sent correctly.

Example:

```json
{
	"auth": {
		"enable": true
	}
}
```

### database

Use the `database` key to specify the database the app should store its data in. The `url` should be a valid SQLite or PostgreSQL database url.

```json
{
	"database": {
		"url": "postgres://postgres:password@host:port/database",
		"max_connections": 5
	}
}
```

### host

Use the `host` key to specify the host the server will bind to. The app will prefer the `HOST` environment variable if it is set. The default value is `0.0.0.0`.

### port

Use the `port` key to specify the port the server will bind to. The app will prefer the `PORT` environment variable if it is set. The default value is `8080`.

### smtp

Use the `smtp` key to configure the SMTP server used to send authentication and alert emails.

```json
{
	"smtp": {
		"host": "smtp.tangram.dev",
		"username": "username",
		"password": "password"
	}
}
```

### storage

Use the `storage` key to configure storage for `.tangram` model files.

#### Local

The `local` storage type stores data in the local filesystem on the server the app is running on.

```json
{
	"storage": {
		"type": "local",
		"path": "path/to/storage/dir"
	}
}
```

#### S3

The `s3` storage type stores data in an S3-compatible cloud storage bucket.

```json
{
	"storage": {
		"type": "s3",
		"access_key": "accessKey",
		"secret_key": "secretKey",
		"endpoint": "endpoint",
		"bucket": "bucket",
		"region": "region",
		"cache_path": "path/to/local/cache/dir"
	}
}
```

#### url

Use the `url` key to specify the URL at which the app is accessible to users. This is used for links in invitation emails.

```json
{
	"url": "https://app.tangram.dev"
}
```
