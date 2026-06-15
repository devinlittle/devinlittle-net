# The GradeGetter Stack

GradeGetter consists of:
* GradeGetter:
  - securely handles users session_tokens in order to scrape grades and provide them to a client as JSON stored as encrypted blobs
* TokenGetter:
  - extracts session tokens from user provided credentials
* GradeGetter_Backend:
  - securely provide an interface in which clients interact with to fetch grades, provide credentials, bootstrap, etc.
