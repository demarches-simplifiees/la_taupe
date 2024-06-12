// const http = require('http');
// const fs = require('fs');
// const path = require('path');

const express = require('express');
const app = express();

app.use(express.static('../2ddoc'));

app.get('/500', function(req, res){
  res.writeHead(500, {'Content-Type': 'text/plain'});
  res.end('KO: 500');
});

app.get('/too_big', function(req, res){
  res.writeHead(200, {'Content-Type': 'text/plain'});
  let bigString = '';
  for (let i = 0; i < 10 * 1024 * 1024 +1 ; i++) {
    bigString += 'a';
  }
  res.end(bigString);
});

app.listen(3333);
