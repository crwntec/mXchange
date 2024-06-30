import hmac
import hashlib

# Given values
challenge = "<1719667092.mx24.144@example.net>"
passphrase = "pop"

# Ensure correct encoding
encoded_challenge = challenge.encode()
encoded_passphrase = passphrase.encode()

# Print encoded values for debugging
print(f"Encoded Challenge: {encoded_challenge}")
print(f"Encoded Passphrase: {encoded_passphrase}")

# Compute HMAC-MD5
hash_obj = hmac.new(encoded_passphrase, encoded_challenge, hashlib.md5)
computed_hash = hash_obj.hexdigest()

print(f"Computed HMAC-MD5 Hash: {computed_hash}")
