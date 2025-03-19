#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <strings.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <pthread.h>
#include "include/zlgcan.h"
#include "include/USBCANFD800U.h"
// #include <iostream>

#define msleep(ms)  usleep((ms)*1000)
#define min(a,b)  (((a) < (b)) ? (a) : (b))

#define CANFD_TEST  1
#define MERGE_TEST  1//合并接收

#define MAX_CHANNELS  2//测试通道
#define CHECK_POINT  200
#define RX_WAIT_TIME  100
#define RX_BUFF_SIZE  1000

unsigned gDevType = 0;
unsigned gDevIdx = 0;
unsigned gChMask = 0;
unsigned gTxType = 0;
unsigned gTxSleep = 0;
unsigned gTxFrames = 0;
unsigned gTxCount = 0;
unsigned gDebug = 0;

unsigned s2n(const char *s)
{
    unsigned l = strlen(s);
    unsigned v = 0;
    unsigned h = (l > 2 && s[0] == '0' && (s[1] == 'x' || s[1] == 'X'));
    unsigned char c;
    unsigned char t;
    if (!h) return atoi(s);
    if (l > 10) return 0;
    for (s += 2; c = *s; s++) {
        if (c >= 'A' && c <= 'F') c += 32;
        if (c >= '0' && c <= '9') t = c - '0';
        else if (c >= 'a' && c <= 'f') t = c - 'a' + 10;
        else return 0;
        v = (v << 4) | t;
    }
    return v;
}

typedef struct {
    CHANNEL_HANDLE channel_handle; 
    unsigned channel;// channel index, 0~7
    unsigned stop; // stop RX-thread
    unsigned total; // total received
    unsigned error; // error(s) detected
} THREAD_CTX;

void* rx_thread(void *data)
{
    THREAD_CTX *ctx = (THREAD_CTX *)data;
    ctx->total = 0; // reset counter
#if MERGE_TEST
    ZCANDataObj zlg_buff[RX_BUFF_SIZE] = {0};
#else
    #if CANFD_TEST
        ZCAN_ReceiveFD_Data zlg_buff[RX_BUFF_SIZE] = {0};
    #else
        ZCAN_Receive_Data zlg_buff[RX_BUFF_SIZE];
    #endif
#endif

    int cnt; // current received

    unsigned check_point = 0;
    while (!ctx->stop && !ctx->error)
    {
#if MERGE_TEST
        cnt = ZCAN_ReceiveData(ctx->channel_handle, zlg_buff, RX_BUFF_SIZE, RX_WAIT_TIME);
        if (!cnt)
            continue;
        for (int i = 0; i < cnt; i++) {
            if (zlg_buff[i].dataType != ZCAN_DT_ZCAN_CAN_CANFD_DATA)
            {
                continue;
            }
            fprintf(stderr,"%s recv ID: 0x%X \tDLC: %d \t data:",__FUNCTION__, zlg_buff[i].data.zcanCANFDData.frame.can_id,
             zlg_buff[i].data.zcanCANFDData.frame.len);
            for (int j = 0; j < zlg_buff[i].data.zcanCANFDData.frame.len; j++)
            {
                fprintf(stderr,"%02X ", zlg_buff[i].data.zcanCANFDData.frame.data[j]);
            }
            fprintf(stderr,"\n");
        }
#else
    #if CANFD_TEST  
            cnt = ZCAN_ReceiveFD(ctx->channel_handle, zlg_buff, RX_BUFF_SIZE, RX_WAIT_TIME);
            if (!cnt)
                continue;

            for (int i = 0; i < cnt; i++) {
                fprintf(stderr,"%s recv ID: 0x%X \tDLC: %d \t data:",__FUNCTION__, zlg_buff[i].frame.can_id, zlg_buff[i].frame.len);
                for (int j = 0; j < zlg_buff[i].frame.len; j++)
                {
                    fprintf(stderr,"%02X ", zlg_buff[i].frame.data[j]);
                }
                fprintf(stderr,"\n");
            }
    #else
            cnt = ZCAN_Receive(ctx->channel_handle, zlg_buff, RX_BUFF_SIZE, RX_WAIT_TIME);
            if (!cnt)
                continue;

            for (int i = 0; i < cnt; i++) {
                fprintf(stderr,"%s recv ID: 0x%X \tDLC: %d \t data:",__FUNCTION__, zlg_buff[i].frame.can_id, zlg_buff[i].frame.can_dlc);
                for (int j = 0; j < zlg_buff[i].frame.can_dlc; j++)
                {
                    fprintf(stderr,"%02X ", zlg_buff[i].frame.data[j]);
                }
                fprintf(stderr,"\n");
            }
    #endif
#endif
        if (ctx->error) break;

        ctx->total += cnt;
        if (ctx->total / CHECK_POINT >= check_point) {
            fprintf(stderr,"%s CAN%d: RX: %d frames received & verified\n", __FUNCTION__, ctx->channel, ctx->total);
            check_point++;
        }
    }

    fprintf(stderr,"%s CAN%d: RX: rx-thread terminated, %d frames received & verified: %s\n",__FUNCTION__,
        ctx->channel, ctx->total, ctx->error ? "error(s) detected" : "no error");

    pthread_exit(0);
    return NULL;
}

void* tx_thread(void *data)
{
    THREAD_CTX *ctx = (THREAD_CTX *)data;
    UINT port = ctx->channel;
    fprintf(stderr,"%s port = %d\n",__FUNCTION__,port);
    time_t tm1, tm2;
    unsigned tx;
    int j;

    ctx->total = 0; // reset counter
#if MERGE_TEST
    int msgsz = sizeof(ZCANDataObj);
    ZCANDataObj *trans_data = (ZCANDataObj *)malloc(msgsz * gTxFrames);
#else
    #if CANFD_TEST
        int msgsz = sizeof(ZCAN_TransmitFD_Data);
        ZCAN_TransmitFD_Data *trans_data = (ZCAN_TransmitFD_Data *)malloc(msgsz * gTxFrames);
        
    #else
        int msgsz = sizeof(ZCAN_Transmit_Data);
        ZCAN_Transmit_Data *trans_data = (ZCAN_Transmit_Data *)malloc(msgsz * gTxFrames);
    #endif
#endif


if (trans_data) {
    memset(trans_data, 0, msgsz * gTxFrames);
    time(&tm1);
    for (tx = 0; !ctx->error && tx < gTxCount; tx++) {
#if MERGE_TEST
    int msgsz = sizeof(ZCANDataObj);
    ZCANDataObj *trans_data = (ZCANDataObj *)malloc(msgsz * gTxFrames);
    for (int i = 0; i < gTxFrames; ++i){
        memset(&trans_data[i], 0, sizeof(ZCANDataObj));
        trans_data[i].chnl = port;
        trans_data[i].dataType =  ZCAN_DT_ZCAN_CAN_CANFD_DATA;
        trans_data[i].data.zcanCANFDData.frame.can_id = i;    
        trans_data[i].data.zcanCANFDData.frame.len = 8;                    
        trans_data[i].data.zcanCANFDData.flag.unionVal.transmitType = 0;             
        trans_data[i].data.zcanCANFDData.flag.unionVal.txEchoRequest = 1;           
        trans_data[i].data.zcanCANFDData.flag.unionVal.frameType = 1;//CANFD       
        // trans_data[i].data.zcanCANFDData.flag.unionVal.txDelay = ZCAN_TX_DELAY_UNIT_MS;                      
        for (int j = 0; j < trans_data[i].data.zcanCANFDData.frame.len; ++j) {        
            trans_data[i].data.zcanCANFDData.frame.data[j] = j;
        }
    }
    if (gTxFrames != ZCAN_TransmitData(ctx->channel_handle, trans_data, gTxFrames))
    {
        // printf("CAN%d TX failed: ID=%08x\n", port, buff->hdr.id);
        fprintf(stderr,"CAN%d TX failed: ID=%08x\n", port, trans_data->data.zcanCANFDData.frame.can_id);
        ctx->error = 1;
        break;
    }
#else    
    #if CANFD_TEST
            for (int i = 0; i < gTxFrames; ++i){
                memset(&trans_data[i], 0, sizeof(ZCAN_TransmitFD_Data));
                trans_data[i].frame.can_id = i;						
                trans_data[i].frame.len = 8;					
                trans_data[i].transmit_type = 2;					
                memset(trans_data[i].frame.data, i, trans_data[i].frame.len);
            }
            if (gTxFrames != ZCAN_TransmitFD(ctx->channel_handle, trans_data, gTxFrames))
    #else
            for (int i = 0; i < gTxFrames; ++i){
                memset(&trans_data[i], 0, sizeof(ZCAN_Transmit_Data));
                trans_data[i].frame.can_id = i;						
                trans_data[i].frame.can_dlc = 8;					
                trans_data[i].transmit_type = 2;					
                memset(trans_data[i].frame.data, i, trans_data[i].frame.can_dlc);
            }
            if (gTxFrames != ZCAN_Transmit(ctx->channel_handle, trans_data, gTxFrames))
    #endif
            {
                // printf("CAN%d TX failed: ID=%08x\n", port, buff->hdr.id);
                fprintf(stderr,"CAN%d TX failed: ID=%08x\n", port, trans_data->frame.can_id);
                ctx->error = 1;
                break;
            }
#endif
            ctx->total += gTxFrames;
            if (gTxSleep) msleep(gTxSleep);
        }
        time(&tm2);
        free(trans_data);
    }
    else ctx->error = -1;

    if (!ctx->error) {
        fprintf(stderr,"CAN%d: TX: %d frames sent, %ld seconds elapsed\n",
            port, gTxFrames * gTxCount, tm2 - tm1);
        if (tm2 - tm1)
            fprintf(stderr,"CAN%d: TX: %ld frames/second\n", port, gTxFrames * gTxCount / (tm2 - tm1));
    }

    pthread_exit(0);
    return NULL;
}

int test(void * device_handle)
{
    union UnionCANFDBaudBitField
    {
        struct
        {
            __uint32_t    nTSEG1 : 8;     // ABIT: 1-64  DBIT: 1-32
            __uint32_t    nTSEG2 : 7;     // ABIT: 1-32  DBIT: 1-8
            __uint32_t    nSJW : 7;       // ABIT: 1-32  DBIT: 1-8
            __uint32_t    nBRP : 10;      // ABIT: 1-256 DBIT: 1-256
        }               unionValue;
        __uint32_t        nRawValue;
    }baud;

    // ----- device info --------------------------------------------------

    ZCAN_DEVICE_INFO info;
    memset(&info, 0, sizeof(info));
    if(STATUS_OK == ZCAN_GetDeviceInf(device_handle, &info))
    {
        char str_Serial_Num[21];
        char str_hw_Type[41];
        memcpy(str_Serial_Num, info.str_Serial_Num, 20);
        memcpy(str_hw_Type, info.str_hw_Type, 40);
        str_Serial_Num[20] = '\0';
        str_hw_Type[40] = '\0';
        printf("HWV=0x%04x, FWV=0x%04x, DRV=0x%04x, API=0x%04x, IRQ=0x%04x, CHN=0x%02x, SN=%s, ID=%s\n",
            info.hw_Version, info.fw_Version, info.dr_Version, info.in_Version, info.irq_Num, info.can_Num, str_Serial_Num, str_hw_Type);
    }else{
         printf("ZCAN_GetDeviceInf failed!\n");
    }

    // ----- init & start -------------------------------------------------

    ZCAN_CHANNEL_INIT_CONFIG init;
    init.can_type = 1;
    init.canfd.mode = 0; //0-正常模式 1-只听模式
    baud.unionValue.nTSEG1 = 31;
    baud.unionValue.nTSEG2 = 8;
    baud.unionValue.nSJW = 8;
    baud.unionValue.nBRP = 1;
    init.canfd.abit_timing = baud.nRawValue;
    baud.unionValue.nTSEG1 = 5;
    baud.unionValue.nTSEG2 = 2;
    baud.unionValue.nSJW = 2;
    baud.unionValue.nBRP = 1;
    init.canfd.dbit_timing = baud.nRawValue;

    int i;
    CHANNEL_HANDLE ch[MAX_CHANNELS] = {};
    for (i = 0; i < MAX_CHANNELS; i++) {
        ch[i] = ZCAN_InitCAN(device_handle, i, &init);
        if (ch[i] == NULL) {
            printf("ZCAN_InitCAN(%d) failed\n", i);
            return 0;
        }
        printf("ZCAN_InitCAN(%d) succeeded\n", i);

        // 使能通道终端电阻
        uint32_t val = 1;
        if (0 == ZCAN_SetReference(gDevType, gDevIdx, i, SETREF_ENABLE_INTERNAL_RESISTANCE, &val))
        {
            printf("enable terminal chn[%d] resistance failed\n", i);
            return 0;
        }else{
            printf("enable terminal chn[%d] resistance success\n", i);
        }
        if (!ZCAN_StartCAN(ch[i])) {
            printf("ZCAN_StartCAN(%d) failed\n", i);
            return 0;
        }
        printf("ZCAN_StartCAN(%d) succeeded\n", i);
    }

  // ----- set merge --------------------------------------------------
#if MERGE_TEST
    int isMerge = 1;
    int nRet = ZCAN_SetReference(gDevType, gDevIdx, 0, SETREF_SET_DATA_RECV_MERGE, &isMerge);
    if (nRet)
    {
        printf("Set Merge succeeded!\n");
    }
    else
    {
        printf("Set Merge failed!\n");
    }
#endif
    // ----- create RX-threads --------------------------------------------
#if MERGE_TEST
    THREAD_CTX rx_ctx;
    pthread_t rx_threads;
    rx_ctx.channel_handle = ch[0];
    rx_ctx.channel = 0;
    rx_ctx.stop = 0;
    rx_ctx.total = 0;
    rx_ctx.error = 0;
    pthread_create(&rx_threads, NULL, rx_thread, &rx_ctx);
#else
    THREAD_CTX rx_ctx[MAX_CHANNELS];
    pthread_t rx_threads[MAX_CHANNELS];
    for (i = 0; i < MAX_CHANNELS; i++) {
        rx_ctx[i].channel_handle = ch[i];
        rx_ctx[i].channel = i;
        rx_ctx[i].stop = 0;
        rx_ctx[i].total = 0;
        rx_ctx[i].error = 0;
        pthread_create(&rx_threads[i], NULL, rx_thread, &rx_ctx[i]);
    }
#endif
    // ----- wait --------------------------------------------------------

    printf("<ENTER> to start TX: %d*%d frames/channel ...\n", gTxFrames, gTxCount);
    getchar();

    // ----- start transmit -----------------------------------------------

    THREAD_CTX tx_ctx[MAX_CHANNELS];
    pthread_t tx_threads[MAX_CHANNELS];
    for (i = 0; i < MAX_CHANNELS; i++) {
        tx_ctx[i].channel_handle = ch[i];
        tx_ctx[i].channel = i;
        tx_ctx[i].stop = 0;
        tx_ctx[i].total = 0;
        tx_ctx[i].error = 0;
        pthread_create(&tx_threads[i], NULL, tx_thread, &tx_ctx[i]);
    }

    // ----- stop TX & RX -------------------------------------------------

    int err = 0;
    for (i = 0; i < MAX_CHANNELS; i++) {
        pthread_join(tx_threads[i], NULL);
        if (tx_ctx[i].error)
            err = 1;
    }

    sleep(2);
    printf("<ENTER> to stop RX ...\n");
    getchar();
#if MERGE_TEST
        rx_ctx.stop = 1;
        pthread_join(rx_threads, NULL);
        if (rx_ctx.error)
            err = 1;
#else
    for (i = 0; i < MAX_CHANNELS; i++) {
        rx_ctx[i].stop = 1;
        pthread_join(rx_threads[i], NULL);
        if (rx_ctx[i].error)
            err = 1;
    }
#endif

    // ----- report -------------------------------------------------------

    printf(err ? "error(s) detected, test failed\n" : "test succeeded\n");
    return !err;
}

int main(int argc, char* argv[])
{
    if (argc < 5) {
        printf("test [DevType] [DevIdx] [TxFrames] [TxCount]\n"
            "    example: test 59 0 10 1000 \n"
            "                  |  | |    |1000(count)\n"
            "                  |  | 10 frames once\n"
            "                  |  |Card0\n"
            "                  |59-usbcanfd-800u....\n"
            );
        return 0;
    }
    gDevType = s2n(argv[1]);
    gDevIdx = s2n(argv[2]);
    gTxFrames = s2n(argv[3]);
    gTxCount = s2n(argv[4]);
    printf("DevType=%d, DevIdx=%d, TxFrames=0x%08x(%d), TxCount=0x%08x(%d)\n",
        gDevType, gDevIdx, gTxFrames, gTxFrames, gTxCount, gTxCount);

    void * device_handle =  ZCAN_OpenDevice(gDevType, gDevIdx, 0);
    if (NULL == device_handle) {
        printf("ZCAN_OpenDevice failed\n");
        return 0;
    }
    printf("ZCAN_OpenDevice succeeded\n");

    test(device_handle);

    ZCAN_CloseDevice(device_handle);
    printf("ZCAN_CloseDevice\n");
    return 0;
}

