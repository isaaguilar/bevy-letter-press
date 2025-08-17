#!/bin/bash 
set -o errexit
set -o nounset
set -o pipefail

source .private/env

docker buildx build --platform linux/amd64 --load -t wack-a-weed-leaderboard:dev .

 docker run \
    -it \
    --rm \
    -v ~/.aws:/root/.aws \
    -e AWS_PROFILE=$AWS_PROFILE \
    wack-a-weed-leaderboard:dev \
    cargo lambda deploy --iam-role arn:aws:iam::"$AWS_ACCOUNT_ID":role/$IAM_ROLE wack-a-weed-leaderboard-lambda
    ## Instead of
    
    # TODO 
    #cargo lambda build -#-release --target x86_64-unknown-linux-gnu

    ##
    ## Then move 
    # target/lambda/dino-leaderboard-lambda/bootstrap
    ## Copy it to zip
    # zip lambda.zip bootstrap
    #
    # Upload it with terraform