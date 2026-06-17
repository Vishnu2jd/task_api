## Seed users

curl -i -X POST http://127.0.0.1:3000/seed/users

HTTP/1.1 200 OK
content-length: 0


## Get latest 2FA code (dev)

curl -i http://127.0.0.1:3000/dev/email-logs/latest

HTTP/1.1 200 OK
content-type: application/json
content-length: 24

{"latest_code":"709000"}


## Verify 2FA (admin)

curl -i -X POST http://127.0.0.1:3000/auth/verify-2fa \
  -H "Content-Type: application/json" \
  -d '{"login_challenge_id":"chal_a2be186373c94d3aa57a707083798aeb","code":"709000"}'

HTTP/1.1 200 OK
content-type: application/json
content-length: 172

{"token":"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZG1pbkBleGFtcGxlLmNvbSIsInJvbGUiOiJhZG1pbiIsImV4cCI6MTc4MTc3NzQwMH0.angUa6CIgJ6y-QnAricKL60QIej9D3UlMPdeA7B5iQk"}


## Create task: Clean Aston Martin

curl -i -X POST http://127.0.0.1:3000/tasks \
  -H "Authorization: Bearer <ADMIN_TOKEN>" \
  -H "Content-Type: application/json" \
  -d '{"title":"Clean Aston Martin", "description":"Detail wash", "priority":"high"}'

HTTP/1.1 201 Created
content-type: application/json
content-length: 90

{"id":"1","title":"Clean Aston Martin","status":"todo","priority":"high","assigned_to":""}


## Create task: Order Walther PPK

curl -i -X POST http://127.0.0.1:3000/tasks \
  -H "Authorization: Bearer <ADMIN_TOKEN>" \
  -H "Content-Type: application/json" \
  -d '{"title":"Order Walther PPK", "description":"Equipment check", "priority":"medium"}'

HTTP/1.1 201 Created
content-type: application/json
content-length: 91

{"id":"2","title":"Order Walther PPK","status":"todo","priority":"medium","assigned_to":""}


## Create task: Retrieve microfilm

curl -i -X POST http://127.0.0.1:3000/tasks \
  -H "Authorization: Bearer <ADMIN_TOKEN>" \
  -H "Content-Type: application/json" \
  -d '{"title":"Retrieve microfilm", "description":"Infiltration task", "priority":"low"}'

HTTP/1.1 201 Created
content-type: application/json
content-length: 89

{"id":"3","title":"Retrieve microfilm","status":"todo","priority":"low","assigned_to":""}


## Create task: Encrypt communications

curl -i -X POST http://127.0.0.1:3000/tasks \
  -H "Authorization: Bearer <ADMIN_TOKEN>" \
  -H "Content-Type: application/json" \
  -d '{"title":"Encrypt communications", "description":"Network setup", "priority":"high"}'

HTTP/1.1 201 Created
content-type: application/json
content-length: 94

{"id":"4","title":"Encrypt communications","status":"todo","priority":"high","assigned_to":""}


## Create task: Rendezvous with Q

curl -i -X POST http://127.0.0.1:3000/tasks \
  -H "Authorization: Bearer <ADMIN_TOKEN>" \
  -H "Content-Type: application/json" \
  -d '{"title":"Rendezvous with Q", "description":"Tech updates", "priority":"medium"}'

HTTP/1.1 201 Created
content-type: application/json
content-length: 91

{"id":"5","title":"Rendezvous with Q","status":"todo","priority":"medium","assigned_to":""}


## Login (staff)

curl -i -X POST http://127.0.0.1:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"jamesbond@example.com","password":"doubleseven007"}'

HTTP/1.1 200 OK
content-type: application/json
content-length: 62

{"login_challenge_id":"chal_bde3874ceee948b4bbd30a5b7d7a11dd"}


## Get latest 2FA code (dev)

curl -i http://127.0.0.1:3000/dev/email-logs/latest

HTTP/1.1 200 OK
content-type: application/json
content-length: 24

{"latest_code":"019000"}
