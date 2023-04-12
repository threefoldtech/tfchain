# Workflows

This page contains some documentation on the workflows for this repository.

## build_test

The workflow build_test is used to build and test the code (see build_test.yaml). Notice that the binaries are being cached so that the build process is sped up. Once the binaries are build the pipeline will run both the unit tests and the integration tests. This can take up to 30 minutes. The pipeline is ran on every commit to a PR and also when the PR has been merged with development. PRs should only be merged if the pipeline was green (if all tests passed).

For performance reasons we are using a self hosted runner for running the pipeline. The runner will only run one pipeline at a time which means that all other runs will be queued. As the pipeline is ran on every commit it will thus also queue runs of consecutive pushed commits. We strongly advice to add `[skip ci]` to the commit messages whenever possible (when the run of a pipeline can be skipped). A pipeline can also be canceled [here](https://github.com/threefoldtech/tfchain/actions).

### Docker image

We are using a custom docker image for building and testing the code. You can find the image on our [Docker Hub](https://hub.docker.com/repository/docker/threefolddev/tfchain). The dockerfile build_test.Dockerfile was used to build that image. If the image no longer meets the expectations please follow these steps:

1) Update the dockerfile as required (add what you need)
2) Build the new image (execute the comment with .github/workflows as working directory and make sure to increment the version):
    > docker build -t threefolddev/tfchain:\<VERSION> -f build_test.Dockerfile .
3) Make sure to test the pipeline manually using the docker image that you build
    > docker run --it -rm threefolddev/tfchain:\<VERSION> \
    > // clone the repository, build and test tfchain
4) Upload the image (you will need the credentials):
    > docker login
    > docker push threefolddev/tfchain:\<VERSION>
5) Update the pipeline on the repository <https://github.com/threefoldtech/tfchain> under .github/workflows to use this new image
6) Push your changes, create a PR and let the pipeline run. If it fails due to changes to the image you should start over the steps.
7) Once the PR is landed please tag the repository so that we can trace back the dockerfile given the version:
    > git tag -a dockerfile-build-v\<VERSION> -m "new dockerfile version \<VERSION>"

## Publishing of a docker image

[publish_container_image.yml](./publish_container_image.yml) builds the docker container of a tfchain node and publishes it to the github container repository on a release.
