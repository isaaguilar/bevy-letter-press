provider "aws" {
  region = "us-east-1"
}

resource "aws_dynamodb_table" "wack_a_weed_leaderboard" {
  name           = "wack_a_weed_leaderboard"
  billing_mode   = "PROVISIONED"
  read_capacity  = 1
  write_capacity = 1

  hash_key  = "name"
  range_key = "score"

  global_secondary_index {
    name            = "level-index"
    hash_key        = "level"
    projection_type = "ALL"
    read_capacity   = 1
    write_capacity  = 1
  }

  attribute {
    name = "name"
    type = "S"
  }

  attribute {
    name = "level"
    type = "N"
  }

  attribute {
    name = "score"
    type = "N"
  }

  tags = {
    Environment = "dev"
    Project     = "wack-a-weed"
  }
}

resource "aws_iam_role" "lambda_exec_role" {
  name = "wack_a_weed_leaderboard_lambda_dynamodb_access"

  assume_role_policy = jsonencode({
    Version = "2012-10-17",
    Statement = [{
      Action = "sts:AssumeRole",
      Principal = {
        Service = "lambda.amazonaws.com"
      },
      Effect = "Allow",
      Sid    = ""
    }]
  })
}

resource "aws_iam_policy" "dynamodb_access" {
  name        = "wack_a_weed_dynamodb_policy"
  description = "Allow Lambda to access DynamoDB leaderboard table"

  policy = jsonencode({
    Version = "2012-10-17",
    Statement = [
      {
        Effect = "Allow",
        Action = [
          "dynamodb:PutItem",
          "dynamodb:Scan",
          "dynamodb:DescribeTable"
        ],
        Resource = aws_dynamodb_table.wack_a_weed_leaderboard.arn
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "lambda_policy_attach" {
  role       = aws_iam_role.lambda_exec_role.name
  policy_arn = aws_iam_policy.dynamodb_access.arn
}
