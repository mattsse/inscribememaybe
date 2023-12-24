#!/bin/bash

# Check if the number of executions is passed as an argument
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <number of executions>"
    exit 1
fi

# Number of times the command should be executed
num_executions=$1

# Command to be executed
command='$ inscribememaybe mint "{"p":"fair-20","op":"mint","tick":"brr","amt":"1000"}" --private-key "your_private_key" --rpc-url <rpc-url> --transactions 10'

# Loop and execute the command
for (( i=1; i<=num_executions; i++ ))
do
    eval $command
    echo "Execution $i of $num_executions complete"

    # Sleep for 1 second
    sleep 1
done

echo "All executions completed."
