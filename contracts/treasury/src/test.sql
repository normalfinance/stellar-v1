tokens_per_owner AS (
    SELECT md5(contract_address || owner_address) as id,
        contract_address,
        owner_address,
        token_id,
        SUM(value) AS balance
    FROM ledger
    WHERE owner_address <> '0x0000000000000000000000000000000000000000'
    GROUP BY contract_address,
        owner_address,
        token_id
)
SELECT md5(contract_address || owner_address) as id,
    contract_address,
    owner_address,
    COUNT(*) AS tokens_held,
    min(block_number) as first_updated,
    max(block_number) as last_updated
FROM tokens_per_owner
WHERE balance > 0
GROUP BY contract_address,
    owner_address,
    ORDER BY tokens_held DESC;