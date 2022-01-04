#!/usr/bin/env bash
set -e
# numbers of investors
I_NUM=5
# numbers of devs
D_NUM=10

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

# INVESTORS (ONLY TIDEs)
for i in $(seq 1 $I_NUM); do
	INVESTORS+="(CurrencyId::Tide,\n$(generate_address_and_account_id investors$i stash)\n// 1000 TIDE\n1_000_000_000_000_000),\n"
done

# DEVS (TIDEs & CURRENCIES)
for i in $(seq 1 $D_NUM); do
	ADDRESS=$(generate_address_and_account_id dev$i stash)
	DEVS+="(CurrencyId::Tide,\n$ADDRESS\n// 1_000_000_000 TIDE\n1_000_000_000_000_000_000_000),\n"
	DEVS+="(CurrencyId::Wrapped(1),\n$ADDRESS\n// 1_000_000_000 USDT\n1_000_000_000_000_000),\n"
	DEVS+="(CurrencyId::Wrapped(2),\n$ADDRESS\n// 1_000_000_000 USDC\n1_000_000_000_000_000),\n"
	DEVS+="(CurrencyId::Wrapped(100),\n$ADDRESS\n// 1_000_000_000 BTC\n100_000_000_000_000_000),\n"
	DEVS+="(CurrencyId::Wrapped(1000),\n$ADDRESS\n// 1_000_000_000 ETH\n1_000_000_000_000_000_000_000),\n"
done

# FAUCET
FAUCET="(CurrencyId::Tide,\n$(generate_address_and_account_id faucet controller)\n// 10_000 TIDE\n10_000_000_000_000_000),\n"

printf "\nvec![\n"
printf "// faucet\n$FAUCET// investors\n$INVESTORS\n// devs\n$DEVS]"
