#!/usr/bin/env bash

ROOT=`dirname "$0"`

# Make pushd/popd silent.
pushd () {
	command pushd "$@" > /dev/null
}

popd () {
	command popd "$@" > /dev/null
}
