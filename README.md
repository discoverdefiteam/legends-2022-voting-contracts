# Legends Event Voting Contract

## Instantiating the contract

```json
{
  "admins_cw4_group": "juno1....",
  "makers_cw4_group": "juno1...."
}
```

## Executing Contract

Below are the required messages for each endpoint.

### Adding Categories

```json
{
  "add_category": {
    "category": "category_1"
  }
}
```

### Adding Entries

```json
{
  "add_entry": {
    "name": "entry_1",
    "category": "category_1",
    "maker_addr": "juno1....",
    "maker_name": "maker_1",
    "breeder": "breeder_1",
    "genetics": "genetics_1",
    "farmer": "farmer_1"
  }
}
```

### Voting

Because votes are between 1.00 and 10.00, they need to be sent to the contract as a string.

Vote examples:

- 5.65 => **565**
- 9.20 => **920**
- 10.00 => **1000**
- 1.00 => **100**

Votes are saved as:
`(entry_id, voter_addr) -> vote` 

```json
{
  "vote": {
    "category": "category_1",
    "entry_id": "juno1....",
    "votes": {
      "look": "920",
      "smell": "280",
      "taste": "670",
      "post_melt": "125"
    }
  }
}
```

## Querying Contract

Below are the required messages for each endpoint.

### Get Categories

```json
{
  "categories": {}
}
```

### Get Entries

```json
{
  "entry": {
    "category": "category_1",
    "start_after": 1, // optional field
    "limit": 10 // optional field
  }
}
```

### Get Single Entry

```json
{
  "entry": {
    "category": "category_1",
    "entry_id": 1
  }
}
```

### Tally Votes

```json
{
  "tally": {
    "entry_id": 1,
    "start_after": "juno1....", // optional field
    "limit": 10 // optional field
  }
}
```