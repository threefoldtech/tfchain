# Volta nodejs, npm, yarn tools manager
curl https://get.volta.sh | bash

# source env variables added by Volta
source ~/.bash_profile || source ~/.profile || source ~/.bashrc || :

volta install node@14
volta install yarn
volta install npx
