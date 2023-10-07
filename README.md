[![progress-banner](https://backend.codecrafters.io/progress/http-server/1a63bb07-bebc-4e1a-822d-541074c7b926)](https://app.codecrafters.io/users/YisusChrist?r=2qF)

This is a starting point for Rust solutions to the
["Build Your Own HTTP server" Challenge](https://app.codecrafters.io/courses/http-server/overview).

[HTTP](https://en.wikipedia.org/wiki/Hypertext_Transfer_Protocol) is the
protocol that powers the web. In this challenge, you'll build a HTTP/1.1 server
that is capable of serving multiple clients.

Along the way you'll learn about TCP servers,
[HTTP request syntax](https://www.w3.org/Protocols/rfc2616/rfc2616-sec5.html),
and more.

**Note**: If you're viewing this repo on GitHub, head over to
[codecrafters.io](https://codecrafters.io) to try the challenge.

# Introduction

Welcome to the Build your own HTTP server challenge!

In this challenge, you'll build a toy HTTP server that's capable of handling GET/POST requests. Along the way, we'll learn about TCP connections, HTTP headers, HTTP verbs, handling multiple connections and more.

# Repository Setup

We've prepared a starter repository with some Rust code for you.

Step 1: Clone the repository.

```sh
git clone https://git.codecrafters.io/0175455b294ede53 codecrafters-http-server-rust && cd codecrafters-http-server-rust
```

Step 2: Push an empty commit.

```sh
git commit --allow-empty -m 'test' && git push origin master
```

# Passing the first stage

The entry point for your HTTP server implementation is in `src/main.rs`. Study
and uncomment the relevant code, and push your changes to pass the first stage:

```sh
git add .
git commit -m "pass 1st stage" # any msg
git push origin master
```

Time to move on to the next stage!

# Stage 2 & beyond

Note: This section is for stages 2 and beyond.

1. Ensure you have `cargo (1.70)` installed locally
1. Run `./your_server.sh` to run your program, which is implemented in
   `src/main.rs`. This command compiles your Rust project, so it might be slow
   the first time you run it. Subsequent runs will be fast.
1. Commit your changes and run `git push origin master` to submit your solution
   to CodeCrafters. Test output will be streamed to your terminal.


# Functionalities implemented for each stage

Here are the functionalities that you'll need to implement for each stage:

## Stage 1: Bind to a port

In this stage, your task is to start a TCP server on port 4221.


## Stage 2: Respond with 200

In this stage, you'll respond to a HTTP request with a 200 OK response.

Your program will need to:

- Accept a TCP connection
- Read data from the connection (we'll get to parsing it in later stages)
- Respond with `HTTP/1.1 200 OK\r\n\r\n` (there are two `\r\n`s at the end)
   - `HTTP/1.1 200 OK` is the [HTTP Status Line](https://developer.mozilla.org/en-US/docs/Web/HTTP/Messages#status_line).
   - \r\n, also known as [CRLF](https://developer.mozilla.org/en-US/docs/Glossary/CRLF), is the end-of-line marker that HTTP uses.
   - The first `\r\n` signifies the end of the status line.
   - The second `\r\n` signifies the end of the response headers section (which is empty in this case).

It's okay to ignore the data received from the connection for now. We'll get to parsing it in later stages.

For more details on the structure of a HTTP response, view the [MDN docs](https://developer.mozilla.org/en-US/docs/Web/HTTP/Messages#http_responses).


## Stage 3: Respond with 404

In this stage, your program will need to extract the path from the HTTP request.

Here's what the contents of a HTTP request look like:

```
GET /index.html HTTP/1.1

Host: localhost:4221
User-Agent: curl/7.64.1
```

- `GET /index.html HTTP/1.1` is the [start line](https://developer.mozilla.org/en-US/docs/Web/HTTP/Messages#start_line).
   - `GET` is the [HTTP method](https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods).
   - `/index.html` is the [path](https://developer.mozilla.org/en-US/docs/Web/HTTP/Messages#path).
   - `HTTP/1.1` is the [HTTP version](https://developer.mozilla.org/en-US/docs/Web/HTTP/Messages#http_versions).
- `Host: localhost:4221` and `User-Agent: curl/7.64.1` are [HTTP headers](https://developer.mozilla.org/en-US/docs/Web/HTTP/Messages#headers).
- Note that all of these lines are separated by `\r\n`, not just `\n`.

In this stage, we'll only focus on extracting the path from the request.

If the path is `/`, you'll need to respond with a `200 OK `response. Otherwise, you'll need to respond with a `404 Not Found` response.


## Stage 4: Respond with content

In this stage, your program will need to respond with a body. In the previous stages we were only sending a status code, no body.

The tester will send you a request of the form `GET /echo/<a-random-string>`.

Your program will need to respond with a `200 OK` response. The response should have a content type of `text/plain`, and it should contain the random string as the body.

As an example, here's a request you might receive:

```
GET /echo/abc HTTP/1.1

Host: localhost:4221
User-Agent: curl/7.64.1
```

And here's the response you're expected to send back:

```
HTTP/1.1 200 OK
Content-Type: text/plain
Content-Length: 3

abc
```

Remember, lines in the response are separated by `\r\n`, not just `\n`.


## Stage 5: Parse headers

In this stage, your program will need to parse HTTP request headers.

The tester will send you a request of the form `GET /user-agent`, and it'll include a `User-Agent` header.

Your program will need to respond with a `200 OK` response. The response should have a content type of `text/plain`, and it should contain the user agent value as the body.

For example, here's a request you might receive:

```
GET /user-agent HTTP/1.1

Host: localhost:4221
User-Agent: curl/7.64.1
```

and here's the response you're expected to send back:

```
HTTP/1.1 200 OK

Content-Type: text/plain
Content-Length: 11

curl/7.64.1
```


## Stage 6: Concurrent connections

Up until now, we've only tested your program against a single connection in each stage.

In this stage, your server will need to handle multiple concurrent connections.

The tester will send you multiple requests at the same time. Your server will need to respond to all of them.


## Stage 7: Get a file

In this stage, your server will need to return the contents of a file.

The tester will execute your program with a `--directory` flag like this:

```sh
./your_server.sh --directory <directory>
```

It'll then send you a request of the form `GET /files/<filename>`.

If `<filename>` exists in `<directory>`, you'll need to respond with a `200 OK` response. The response should have a content type of `application/octet-stream`, and it should contain the contents of the file as the body.

If the file doesn't exist, return a `404 Not Found` response.


## Stage 8: Post a file

In this stage, your server will need to accept the contents of a file in a POST request and save it to a directory.

Just like in the previous stage, the tester will execute your program with a `--directory` flag like this:

```sh
./your_server.sh --directory <directory>
```

It'll then send you a request of the form `POST /files/<filename>`. The request body will contain the contents of the file.

You'll need to fetch the contents of the file from the request body and save it to `<directory>/<filename>`. The response code returned should be `201 Created`.


# Extra functionalities added

- Added mime types autodection for files.
