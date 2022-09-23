# Workflows
This page contains some documentation on the workflows for this repository.

## build_test
The workflow build_test is used to build and test the code (see build_test.yaml). We are using a custom docker image for building and testing the code. You can find the image on our [Docker Hub](https://hub.docker.com/repository/docker/threefolddev/tfchain). The dockerfile build_test.Dockerfile was used to build that image. If the image no longer meets the expectations please follow these steps:

1) Update the dockerfile as required (add what you need)
2) Build the new image (execute the comment with .github/workflows as working directory and make sure to increment the version):
    > docker build -t threefolddev/tfchain:\<VERSION> -f build_test.Dockerfile .
3) Make sure to test the pipeline manually using the docker image that you build
    > docker run --it -rm threefolddev/tfchain:\<VERSION> \
    > // clone the repository, build and test tfchain
4) Upload the image (you will need the credentials):
    > docker login
    > docker push threefolddev/tfchain:\<VERSION>
5) Update the pipeline on the repository https://github.com/threefoldtech/tfchain under .github/workflows to use this new image
6) Push your changes, create a PR and let the pipeline run. If it fails due to changes to the image you should start over the steps.
7) Once the PR is landed please tag the repository so that we can trace back the dockerfile given the version:
    > git tag -a dockerfile-build-v\<VERSION> -m "new dockerfile version \<VERSION>"

