#include "gui.h"

#define MAXSTRINGSIZE 500 // max space for string
#define PORT 4149

SOCKET s;
char* buffer;

//open a socket to send packets to the gui
int guiStart(){
    WSADATA wsa;
    struct sockaddr_in server;

    if (WSAStartup(MAKEWORD(2, 2), &wsa) != 0)
    {
        printf("Failed. Error Code : %d", WSAGetLastError());
        return 1;
    }

    //Create a socket
    if ((s = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP)) == SOCKET_ERROR)
    {
        printf("Could not create socket : %d", WSAGetLastError());
        return 1;
    }

    server.sin_addr.s_addr = htonl(INADDR_LOOPBACK);//localhost, or loopback to same machine
    server.sin_family = AF_INET;
    server.sin_port = htons(PORT);

    //Connect to server
    if (connect(s, (struct sockaddr *)&server, sizeof(server)) < 0)
    {
        puts("connect error");
        return 1;
    }

    //malloc data sending buffer and zero
    buffer = calloc(MAXSTRINGSIZE, sizeof(char));

#if REDIRECTTOFILE
    freopen("output.txt", "w", stdout);
#endif
    return 0;
}

//send data over socket to the gui
//The string literal must be in the form [gui location],[position in location]:[string literal]
void guiPrint(const char* format, ...){
#if ENABLEGUI
    va_list ap;
    //combine format string and args into 1 string
    va_start(ap, format);
    vsnprintf(buffer, MAXSTRINGSIZE, format, ap);
    va_end(ap);

    send(s, buffer, MAXSTRINGSIZE, 0);
#endif
#if ENABLEPRINT
    printf("%s\n",buffer);
#endif
    memset(buffer, 0, MAXSTRINGSIZE);//reset buffer
}

void guiClose(){
    free(buffer);
    closesocket(s);
    WSACleanup();
}

//testing memory mapped file
int mainMEMORYMAP(){
    HANDLE hMapFile;
    LPCTSTR pBuf;

    char name[] = "TESTmapped";

    hMapFile = CreateFileMapping(
        INVALID_HANDLE_VALUE,    // use paging file
        NULL,                    // default security
        PAGE_READWRITE,          // read/write access
        0,                       // maximum object size (high-order DWORD)
        MAXSTRINGSIZE,                // maximum object size (low-order DWORD)
        name);                 // name of mapping object

    pBuf = (LPTSTR)MapViewOfFile(hMapFile,   // handle to map object
        FILE_MAP_ALL_ACCESS, // read/write permission
        0,
        0,
        MAXSTRINGSIZE);

    char testmessage[] = "a first message";
    Sleep(5000);

    CopyMemory((PVOID)pBuf, testmessage, 16);

    return 0;
}