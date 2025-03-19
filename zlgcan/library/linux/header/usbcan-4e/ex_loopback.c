#include <stdio.h>
#include <unistd.h>
#include <string.h>
#include <stdlib.h>
#include <time.h>
#include <zlgcan/zlgcan.h>

//#define USBCAN_8E_U  1

int main(int argc, char *argv[]) {
    DEVICE_HANDLE  dHandle;
    CHANNEL_HANDLE cHandle;
    ZCAN_Receive_Data zrd;
    ZCAN_Transmit_Data ztd;
    ZCAN_DEVICE_INFO  dinfo;
    ZCAN_CHANNEL_ERR_INFO einfo;
    IProperty *property;
    UINT  ch;
    int ret;
    char path[128];
    int i;

    if (argc < 2)
        ch = 0;
    else 
        ch = atoi(argv[1]);
#ifdef USBCAN_8E_U
    dHandle = ZCAN_OpenDevice(ZCAN_USBCAN_8E_U, 0x0, 0);
#else
    dHandle = ZCAN_OpenDevice(ZCAN_USBCAN_4E_U, 0x0, 0);
#endif
    if (dHandle == INVALID_DEVICE_HANDLE) {
        printf("ZCAN_OpenDevice failed\n");
        return -1;
    }
    ret = ZCAN_GetDeviceInf(dHandle, &dinfo);
    if (ret < 0) {
        printf("ZCAN_GetDeviceInf failed\n");
        ZCAN_CloseDevice(dHandle);
        return -1;
    } else {
        printf("ZCAN_GetDeviceInf Version:(%d:%d),Serial: %s\n",  dinfo.hw_Version, 
                                                dinfo.fw_Version ,dinfo.str_Serial_Num);
    }

    property = GetIProperty(dHandle);

#ifdef USBCAN_8E_U
    ZCAN_CHANNEL_INIT_CONFIG  config;
    config.can_type = 0;
    config.can.acc_code = 0x0;
    config.can.acc_mask  = 0xffffffff;
    
    config.can.filter= 0x1;
    config.can.timing0 = 0x0 ;
    config.can.timing1 = 0x14 ;
    config.can.mode = 0;
    cHandle = ZCAN_InitCAN(dHandle, ch, &config);
    if (cHandle == INVALID_CHANNEL_HANDLE) {
        printf("ZCAN_InitCAN failed\n");
        ZCAN_CloseDevice(dHandle);
        return -1;
    }
#else
    cHandle = ZCAN_InitCAN(dHandle, ch, NULL);
    if (cHandle == INVALID_CHANNEL_HANDLE) {
        printf("ZCAN_InitCAN failed\n");
        ZCAN_CloseDevice(dHandle);
        return -1;
    }
	ZCAN_ResetCAN(cHandle);
    snprintf(path, 128, "info/channel/channel_%d/baud_rate", ch);
    property->SetValue(path, "1000000");

    snprintf(path, 128, "info/channel/channel_%d/work_mode", ch);
    property->SetValue(path, "0");

#endif
    ret = ZCAN_StartCAN(cHandle);
    if (ret < 0) {
        printf("ZCAN_StartCAN failed\n");
        ZCAN_CloseDevice(dHandle);
        return -1;
    }

    while (1) {
        memset(&ztd, 0x0, sizeof(ZCAN_Transmit_Data));
        memset(&zrd, 0x0, sizeof(ZCAN_Receive_Data));
        ret = ZCAN_Receive(cHandle, &zrd, 1, 10000); // 10s timeout
        if (ret == 0) {
            printf("ZCAN_Receive Timeout\n");
            break;      
        } else if (ret < 0) {
            printf("ZCAN_Receive failed\n");
            break;
        } else  {
            ztd.transmit_type = 0;
            ztd.frame.can_id = zrd.frame.can_id; 
            ztd.frame.can_dlc = zrd.frame.can_dlc;
            for (i = 0; i < ztd.frame.can_dlc; i++)
                ztd.frame.data[i] = zrd.frame.data[i];
            ret = ZCAN_Transmit(cHandle, &ztd, 1);
            if (ret != 1) {
                printf("ZCAN_Transmit failed\n");
                break;
            }
        }
        ret = ZCAN_ReadChannelErrInfo(cHandle, &einfo);
        if (ret == 1) {
            printf("ZCAN_ReadChannelErrInfo Read a Error(error_code: 0x%x)\n", einfo.error_code);
        }
    }
    ZCAN_ResetCAN(cHandle);
    ReleaseIProperty(property);
    ZCAN_CloseDevice(dHandle);
    return 0;
}
