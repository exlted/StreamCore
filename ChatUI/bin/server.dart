// Copyright (c) 2021, the Dart project authors. Please see the AUTHORS file
// for details. All rights reserved. Use of this source code is governed by a
// BSD-style license that can be found in the LICENSE file.
// https://github.com/dart-lang/samples/blob/master/server/simple/bin/server.dart

import 'dart:convert';
import 'dart:io';

import 'package:shelf/shelf.dart';
import 'package:shelf/shelf_io.dart' as shelf_io;
import 'package:shelf_router/shelf_router.dart' as shelf_router;
import 'package:shelf_static/shelf_static.dart' as shelf_static;
import 'package:shelf_web_socket/shelf_web_socket.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

// #2. Connect chat system to RabbitMQ setup w/ Twitch Ingest
// #3. Implement front-end customizability
//      - User only define JS/CSS
//      - Want interoperability w/ StreamElements and/or StreamLabs

Future main() async {
  // If the "PORT" environment variable is set, listen to it. Otherwise, 8080.
  // https://cloud.google.com/run/docs/reference/container-contract#port
  final port = int.parse(Platform.environment['PORT'] ?? '8080');

  // See https://pub.dev/documentation/shelf/latest/shelf/Cascade-class.html
  final cascade = Cascade()
      // Handle upgrading all websocket requests
      .add(_websocket)
      // First, serve files from the 'public' directory
      .add(_staticHandler)
      // If a corresponding file is not found, send requests to a `Router`
      .add(_router);

  // See https://pub.dev/documentation/shelf/latest/shelf_io/serve.html
  final server = await shelf_io.serve(
    // See https://pub.dev/documentation/shelf/latest/shelf/logRequests.html
    logRequests()
        // See https://pub.dev/documentation/shelf/latest/shelf/MiddlewareExtensions/addHandler.html
        .addHandler(cascade.handler),
    InternetAddress.anyIPv4, // Allows external connections
    port,
  );

  print('Serving at http://${server.address.host}:${server.port}');
  print('Serving at ws://${server.address.host}:${server.port}');
}

// Serve files from the file system.
final _staticHandler =
    shelf_static.createStaticHandler('public', defaultDocument: 'index.html');

// Router instance to handler requests.
final _router = shelf_router.Router()
  ..get('/helloworld', _helloWorldHandler)
  ..get(
    '/time',
    (request) => Response.ok(DateTime.now().toUtc().toIso8601String()),
  )
  ..get('/sum/<a|[0-9]+>/<b|[0-9]+>', _sumHandler);

class ChatParticipant {
  WebSocketChannel socket;
  int id;

  ChatParticipant(WebSocketChannel _socket, int _id)
      : socket = _socket,
        id = _id {}
}

// function when ChatParticipants disconnect to remove them from the active list
// function to get called when ChatParticipants send a message

int _lastParticipantID = 0;
int nextParticipantID() {
  return _lastParticipantID++;
}

final _participantList = <ChatParticipant>[];
void registerChatParticipant(ChatParticipant newMember) {
  _participantList.add(newMember);
}

void removeChatParticipant(ChatParticipant leavingMember) {
  _participantList.remove(leavingMember);
}

void sendChat(ChatParticipant sendingMember, message) {
  for (var participant in _participantList) {
    if (participant != sendingMember) {
      participant.socket.sink.add("$message");
    }
  }
}

final _websocket = webSocketHandler((webSocket) {
  ChatParticipant newMember = ChatParticipant(webSocket, nextParticipantID());
  registerChatParticipant(newMember);

  final subscription = webSocket.stream.listen((message) {
    sendChat(newMember, message);
  });
  subscription.onDone(() {
    removeChatParticipant(newMember);
  });
});

Response _helloWorldHandler(Request request) => Response.ok('Hello, World!');

Response _sumHandler(request, String a, String b) {
  final aNum = int.parse(a);
  final bNum = int.parse(b);
  return Response.ok(
    const JsonEncoder.withIndent(' ')
        .convert({'a': aNum, 'b': bNum, 'sum': aNum + bNum}),
    headers: {
      'content-type': 'application/json',
      'Cache-Control': 'public, max-age=604800',
    },
  );
}

// Notes
// https://api.dart.dev/stable/2.17.0/dart-io/WebSocket-class.html
// https://github.com/achilleasa/dart_amqp/blob/master/example/example.md
// https://github.com/rabbitmq/rabbitmq-tutorials/blob/master/dart/send.dart
// https://github.com/rabbitmq/rabbitmq-tutorials/blob/master/dart/receive.dart
// https://dart.dev/
// https://flutter.dev/
// https://gist.github.com/mitsuoka/2969464
