#!/bin/sh
# SPDX-License-Identifier: Apache-2.0

set -e

CARGO_BIN=$1
MESON_BUILDTYPE=$2
CARGO_TOML=$3
OUTPUT=$4
OUTPUT_DIR=$5
#RUSTC_FLAGS="${RUSTC_FLAGS} print=native-static-libs"
shift 5

case "${MESON_BUILDTYPE}" in
*debug*) CARGO_BUILDTYPE="debug" ;;
*)
	CARGO_BUILDTYPE="release"
	CARGO_BUILD_FLAG="--release"
	;;
esac

${CARGO_BIN} build --manifest-path "${CARGO_TOML}" \
	${CARGO_BUILD_FLAG+"${CARGO_BUILD_FLAG}"} "$@"

eval cp "${CARGO_TARGET_DIR}/${CARGO_BUILDTYPE}/${OUTPUT}" \
	"${OUTPUT_DIR}/"
