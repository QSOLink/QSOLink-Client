# qsolink-tui
JSON/REST based Ham Radio Logging Platform Desktop Client.

This project and repo will serve as the Desktop Client for the operator to log contacts.

QSO Link (*pronounced Q S O Link*), Named after the [Ham Radio Q Codes](https://hamradioprep.com/ham-radio-q-codes/), is intended to be a cross platform ham radio contact logging package.  It comes from a series of frustrations over the many years I've been a ham, where replacing a computer resulted in lost log databases.  Or needing to come up with solutions to sync logs when using a laptop and a desktop, and possibly even a phone depending on the context of how I was operating.  My goal is to use a JSON/REST API server, that can be accessed through a number of different front ends.

### Setup
You will need a `.env` file in the directory that contains the location of your QSOLink server.  Copy the included `.env.sample` and then edit it with your location.
```
$ cp .env.sample .env
```

