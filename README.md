# XChemLab DLS i04-1 Exporter

An export service, providing XChemLab shipment data to the Diamond Light Source ISPyB database.


## Developing Locally (VSCode)

- [Install docker-compose](https://docs.docker.com/compose/install/other/)
- Start the podman socket with `systemctl --user start podman.socket`
- Use the podman socket as the compose host by adding `export DOCKER_HOST=unix:///run/user/$UID/podman/podman.sock` to your `~/.bashrc`
- Install `ms-vscode-remote.remote-containers`
- Set `remote.containers.dockerPath` to `podman`
- Set `remote.containers.dockerComposePath` to your `docker-compose`
- Disable buildkit by adding `export DOCKER_BUILDKIT=0` to your `~/.bashrc`
- Open Development Container
- Start the app, with `cargo run`
