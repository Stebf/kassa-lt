# Kassa-LT

Kassa-LT is a point-of-sale (POS) application to use on a single device.

It uses a Vite frontend and a Rust backend, bundled using Tauri to create a single, cross-plattform application.

![The User Interface of Kassa-LT](docs/assets/pospage_v1.jpg)

## Features

- Sort products into categories and tabs
- Differentiate between cash and card payments
- Quotas to limit the amount of products that can be bought
- Export orders to csv
- Backup the database to a remote target, like a Nextcloud instance
- Admin mode to manage products etc.

### Exports
Show all orders together with a small statistics section and export all orders to csv.

### Tabs
You can create a tabbed interface to easily switch between different sets of products.
### Backup
Kassa-LT supports backing up the database to a remote target, like a Nextcloud instance, which can be configured in the settings. This allows users to easily create backups of their data and store them remotely for safekeeping.  
It supports two different protocols, WebDAV and HTTP(S) PUT. 
WebDAV currently does not work on Android, use HTTP(S) PUT instead.


## Build
You can use the Makefile for common build tasks:
- Run the development environment using `make dev`
- Run the unit tests using `make test`
- More to be expanded

### Release versions
You can create executables according to https://tauri.app/distribute/, right now it is only tested for Windows NullSoftInstaller and Android.
These can be created using `make release-windows` and `make release-android` respectively.
## Development

### WebDAV backup

To test the backup feature locally, run `docker compose up` to start a WebDAV server.
The app needs to be configured with the following settings:

* **WebDAV URL**: `http://localhost:8080`
* **Username**: `alice`
* **Password**: `secret1234`
* **Auth Method**: `Digest`

To check for successful backups, visit [http://localhost:8080](http://localhost:8080) in a web browser.
