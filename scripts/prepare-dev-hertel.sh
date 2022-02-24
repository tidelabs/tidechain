#!/usr/bin/env bash
set -e
# numbers of investors
I_NUM=15
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

# INVESTORS
for i in $(seq 1 $I_NUM); do
	ADDRESS=$(generate_address_and_account_id investor$i stash)
	INVESTORS+="(assets::Asset::Tide.currency_id(),\n$ADDRESS\n// 10_000 TIDE\nassets::Asset::Tide.saturating_mul(10_000)),\n"
	INVESTORS+="(assets::Asset::Tether.currency_id(),\n$ADDRESS\n// 10_000 USDT\nassets::Asset::Tether.saturating_mul(10_000)),\n"
	INVESTORS+="(assets::Asset::USDCoin.currency_id(),\n$ADDRESS\n// 10_000 USDC\nassets::Asset::USDCoin.saturating_mul(10_000)),\n"
	INVESTORS+="(assets::Asset::Bitcoin.currency_id(),\n$ADDRESS\n// 10_000 BTC\nassets::Asset::Bitcoin.saturating_mul(10_000)),\n"
	INVESTORS+="(assets::Asset::Ethereum.currency_id(),\n$ADDRESS\n// 10_000 ETH\nassets::Asset::Ethereum.saturating_mul(10_000)),\n"
done

# DEVS
for i in $(seq 1 $D_NUM); do
	ADDRESS=$(generate_address_and_account_id dev$i stash)
	DEVS+="(assets::Asset::Tide.currency_id(),\n$ADDRESS\n// 1_000_000 TIDE\nassets::Asset::Tide.saturating_mul(1_000_000)),\n"
	DEVS+="(assets::Asset::Tether.currency_id(),\n$ADDRESS\n// 1_000_000 USDT\nassets::Asset::Tether.saturating_mul(1_000_000)),\n"
	DEVS+="(assets::Asset::USDCoin.currency_id(),\n$ADDRESS\n// 1_000_000 USDC\nassets::Asset::USDCoin.saturating_mul(1_000_000)),\n"
	DEVS+="(assets::Asset::Bitcoin.currency_id(),\n$ADDRESS\n// 1_000_000 BTC\nassets::Asset::Bitcoin.saturating_mul(1_000_000)),\n"
	DEVS+="(assets::Asset::Ethereum.currency_id(),\n$ADDRESS\n// 1_000_000 ETH\nassets::Asset::Ethereum.saturating_mul(1_000_000)),\n"

	MARKET_MAKERS+="$ADDRESS\n"
done

# FAUCET
FAUCET="(CurrencyId::Tide,\n$(generate_address_and_account_id faucet controller)\n// 10_000 TIDE\nassets::Asset::Tide.saturating_mul(10_000)),\n"

printf "\nvec![\n$MARKET_MAKERS]"
printf "\nvec![\n"
printf "// faucet\n$FAUCET// investors\n$INVESTORS\n// devs\n$DEVS]"
