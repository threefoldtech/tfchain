# Workflows
This page contains some documentation on the workflows for this repository.

## build_test
The workflow build_test is used to build and test the code (see build_test.yaml). We are using a custom docker image for building and testing the code. You can find the image on our [Docker Hub](https://hub.docker.com/repository/docker/threefolddev/tfchain). The dockerfile build_test.Dockerfile was used to build that image. If the image no longer meets the expectations please follow these steps:
1) Update the dockerfile as required and build an image with it
2) Make sure to test the pipeline manually using the docker image that you build
3) Tag the image (VERSION should be the increment of the last version)
    > docker tag <NAME_OF_IMAGE> threefolddev/tfchain:\<VERSION>
4) Upload the image (you will need the credentials):
    > docker login
    > docker push threefolddev/tfchain:\<VERSION>
5) Update the pipeline on the repository https://github.com/threefoldtech/tfchain under .github/workflows to use this new image
6) Push your changes to the pipeline and let the pipeline run, if it fails due to the new image fix it and start over
7) Once the Dockerfile changes have been merged please tag the repository with the new:
    > git tag -a dockerfile-build-v\<VERSION> -m "new dockerfile version \<VERSION>"
