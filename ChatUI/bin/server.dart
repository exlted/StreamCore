// Copyright (c) 2021, the Dart project authors. Please see the AUTHORS file
// for details. All rights reserved. Use of this source code is governed by a
// BSD-style license that can be found in the LICENSE file.
// https://github.com/dart-lang/samples/blob/master/server/simple/bin/server.dart

import 'dart:developer';
import 'dart:io';

import 'package:shelf/shelf.dart';
import 'package:shelf/shelf_io.dart' as shelf_io;
import 'package:shelf_static/shelf_static.dart' as shelf_static;
import 'package:shelf_web_socket/shelf_web_socket.dart';
import 'package:web_socket_channel/web_socket_channel.dart';
import "package:dart_amqp/dart_amqp.dart";

// #3. Implement front-end customizability
//      - User only define JS/CSS
//      - Want interoperability w/ StreamElements and/or StreamLabs

Future main(List<String> arguments) async {
  RMQChatParticipant rabbitChat = RMQChatParticipant(nextParticipantID());
  registerChatParticipant(rabbitChat);

  ConnectionSettings settings = ConnectionSettings();
  settings.host = Platform.environment['AMPQ_HOST'] ?? "127.0.0.1";
  settings.port = int.parse(Platform.environment['AMPQ_PORT'] ?? "5672");
  settings.authProvider = PlainAuthenticator(
      Platform.environment['AMPQ_USERNAME'] ?? "guest",
      Platform.environment['AMPQ_PASSWORD'] ?? "guest");

  Client client = Client(settings: settings);

  Channel channel = await client
      .channel(); // auto-connect to localhost:5672 using guest credentials
  Exchange exchange = await channel.exchange(
      Platform.environment['EXCHANGE_NAME'] ?? "chat", ExchangeType.TOPIC,
      durable: true);
  Consumer consumer = await exchange.bindPrivateQueueConsumer(["#"]);
  consumer.listen((AmqpMessage message) {
    sendChat(rabbitChat, message.payloadAsString);
  });

  // If the "PORT" environment variable is set, listen to it. Otherwise, 8080.
  // https://cloud.google.com/run/docs/reference/container-contract#port
  final port = 8080;

  // See https://pub.dev/documentation/shelf/latest/shelf/Cascade-class.html
  final cascade = Cascade()
      // Handle upgrading all websocket requests
      .add(_websocket)
      // First, serve files from the 'public' directory
      .add(_staticHandler);

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
    shelf_static.createStaticHandler('public', defaultDocument: 'main.html');

class ChatParticipant {
  WebSocketChannel? socket;
  int id;

  ChatParticipant(WebSocketChannel? _socket, int _id)
      : socket = _socket,
        id = _id {}

  void sendMessage(message) {
    socket?.sink.add("$message");
  }
}

class RMQChatParticipant extends ChatParticipant {
  RMQChatParticipant(int _id) : super(null, _id) {}

  void sendMessage(message) {
    print(message);
  }
}

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
  log(message);
  for (var participant in _participantList) {
    if (participant != sendingMember) {
      participant.sendMessage(message);
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

// Notes
// https://github.com/achilleasa/dart_amqp/blob/master/example/example.md
// https://github.com/rabbitmq/rabbitmq-tutorials/blob/master/dart/send.dart
// https://github.com/rabbitmq/rabbitmq-tutorials/blob/master/dart/receive.dart
