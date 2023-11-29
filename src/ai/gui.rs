// #include <WinSock2.h>
// #include <stdarg.h>
// #include <stdio.h>

use crate::ai::character::PlayerId;
use crate::constants::EnemyId;

pub const LocationMemoryEnemy: i32 = EnemyId;
pub const LocationMemoryPlayer: i32 = PlayerId;
pub static LocationDetection: &'static str = "2";
pub static LocationState: &'static str = "3";
pub static LocationJoystick: &'static str = "4";
pub static LocationHandler: &'static str = "5";

pub const MAXSTRINGSIZE: i32 = 500; // max space for string

// SOCKET s;
// char* buffer;

//open a socket to send packets to the gui
// int guiStart() -> {
//     WSADATA wsa;
//     struct sockaddr_in server;
//
//     if (WSAStartup(MAKEWORD(2, 2), &wsa) != 0)
//     {
//         printf("Failed. Error Code : %d\n", WSAGetLastError());
//         return 1;
//     }
//
//     Create a socket
    // if ((s = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP)) == SOCKET_ERROR)
    // {
    //     printf("Could not create socket : %d\n", WSAGetLastError());
    //     return 1;
    // }
    //
    // server.sin_addr.s_addr = htonl(INADDR_LOOPBACK);//localhost, or loopback to same machine
    // server.sin_family = AF_INET;
    // server.sin_port = htons(PORT);

    //Connect to server
    // if (connect(s, (struct sockaddr *)&server, sizeof(server)) < 0)
    // {
    //     printf("connect error\n");
    //     return 1;
    // }

    //malloc data sending buffer and zero
    // buffer = calloc(MAXSTRINGSIZE, sizeof(char));

// #if ENABLEPRINT && REDIRECTTOFILE
//     freopen("output.txt", "w", stdout);
// #endif
//     return 0;
// }

//send data over socket to the gui
//The string literal must be in the form [gui location],[position in location]:[string literal]
// pub fn guiPrint(const char* format, ...){
// #if ENABLEGUI
//     SYSTEMTIME t;
//     GetSystemTime(&t);

    // va_list ap;
    //combine format string and args into 1 string
    // va_start(ap, format);
    // vsnprintf(buffer, MAXSTRINGSIZE, format, ap);
    // va_end(ap);

    //generate timestamp
    // char timestamp[14];
    // sprintf((char*)&timestamp, ":%02d:%02d:%02d.%03d", t.wHour, t.wMinute, t.wSecond, t.wMilliseconds);
// #endif
// #if ENABLEPRINT
//     printf("[%s]:%s\n", timestamp, buffer);
// #endif
// #if ENABLEGUI
    //copy timestamp to end of packet
    // memcpy(&buffer[strlen(buffer)], timestamp, 13);

    // sendto(s, buffer, MAXSTRINGSIZE, 0, 0, 0);
// #endif
// #if ENABLEGUI
//     memset(buffer, 0, MAXSTRINGSIZE);//reset buffer
// #endif
// }

// void guiClose(){
//     free(buffer);
//     closesocket(s);
//     WSACleanup();
// }