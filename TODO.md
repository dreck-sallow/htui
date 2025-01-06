# CLI:

´´´bash
	htui GET localohost:4000/users 
	htui POST localohost:4000/users -D {name:"Dreck Sallow"}
´´´
htui --url "https://github.com" -method GET -headers "authorization: Bearer [token], x-app-id: otro parsed," -body "name='dikson'&lastName='Aranda'"

cargo run -- --url="https://jsonplaceholder.typicode.com/users/3" get  --headers "x-app-id: 46575832829, Authorization: Bearer json"
