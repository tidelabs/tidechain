#!/usr/bin/env bash

set -e
# numbers of quorum
Q_NUM=3
# numbers of investors
I_NUM=5
# numbers of devs
D_NUM=5

# empty strings
AUTHORITIES=""
QUORUMS=""

# numbers of validators
if [ "$#" -ne 1 ]; then
	V_NUM=3
else
   V_NUM=$1
fi

generate_account_id() {
	subkey inspect ${3:-} ${4:-} "$SECRET//$1//$2" | grep "Account ID" | awk '{ print $3 }'
}

generate_address() {
	subkey inspect ${3:-} ${4:-} "$SECRET//$1//$2" | grep "SS58 Address" | awk '{ print $3 }'
}

generate_public_key() {
	subkey inspect ${3:-} ${4:-} "$SECRET//$1//$2" | grep "Public" | awk '{ print $4 }'
}

generate_address_and_public_key() {
	ADDRESS=$(generate_address $1 $2 $3)
	PUBLIC_KEY=$(generate_public_key $1 $2 $3)

	printf "//$ADDRESS\nhex![\"${PUBLIC_KEY#'0x'}\"].unchecked_into(),"
}

generate_address_and_account_id() {
	ACCOUNT=$(generate_account_id $1 $2 $3)
	ADDRESS=$(generate_address $1 $2 $3)
	if ${4:-false}; then
		INTO="unchecked_into"
	else
		INTO="into"
	fi

	printf "//$ADDRESS\nhex![\"${ACCOUNT#'0x'}\"].$INTO(),"
}

for i in $(seq 1 $V_NUM); do
	AUTHORITIES+="(\n"
	AUTHORITIES+="$(generate_address_and_account_id validator$i stash)\n"
	AUTHORITIES+="$(generate_address_and_account_id validator$i controller)\n"
	AUTHORITIES+="$(generate_address_and_account_id validator$i grandpa '--scheme ed25519' true)\n"
	AUTHORITIES+="$(generate_address_and_account_id validator$i babe '--scheme sr25519' true)\n"
	AUTHORITIES+="$(generate_address_and_account_id validator$i im_online '--scheme sr25519' true)\n"
	AUTHORITIES+="$(generate_address_and_account_id validator$i authority_discovery '--scheme sr25519' true)\n"
	AUTHORITIES+="),\n"
done

printf "// initial authorities\nvec!["
printf "$AUTHORITIES]"

# QUORUMS
for i in $(seq 1 $Q_NUM); do
	QUORUMS+="$(generate_address_and_account_id quorum$i controller)\n"
done

printf "\n\n"
printf "// quorums\nvec![\n"
printf "$QUORUMS]"

# INVESTORS (ONLY TDFYs)
for i in $(seq 1 $I_NUM); do
	INVESTORS+="(CurrencyId::Tdfy,\n$(generate_address_and_account_id investors$i stash)\n// 1_000 TDFY\nassets::Asset::Tdfy.saturating_mul(1_000)),\n"
done

# DEVS (TDFY & CURRENCIES)
for i in $(seq 1 $D_NUM); do
	DEVS+="(CurrencyId::Tdfy,\n$(generate_address_and_account_id dev$i stash)\n// 1_000 TDFY\nassets::Asset::Tdfy.saturating_mul(1_000)),\n"
done

printf "\n\n"
# FAUCET
FAUCET="(CurrencyId::Tdfy,\n$(generate_address_and_account_id faucet controller)\n// 10_000 TDFY\nassets::Asset::Tdfy.saturating_mul(10_000)),\n"

printf "\n// get_stakeholder_tokens_testnet\nvec![\n"
printf "// faucet\n$FAUCET// investors\n$INVESTORS\n// devs\n$DEVS]"
# ORACLE
printf "\n\n"
printf "// oracle\n"
printf "$(generate_address_and_account_id oracle controller)\n"

# ROOT
printf "\n\n"
printf "// root\n"
printf "$(generate_address_and_account_id root controller)\n"

printf "\n\n"