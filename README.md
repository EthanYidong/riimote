# Permissions: 

- Enable bluez dbus permissions: Add to `/etc/dbus-1/system.d/bluetooth.conf`
```
<policy user="__">
    <allow send_destination="org.bluez"/>
    <allow send_interface="org.bluez.Agent1"/>
    <allow send_interface="org.bluez.GattCharacteristic1"/>
    <allow send_interface="org.bluez.GattDescriptor1"/>
    <allow send_interface="org.freedesktop.DBus.ObjectManager"/>
    <allow send_interface="org.freedesktop.DBus.Properties"/>
  </policy>
```
- Enbable udev permissions: Add to `/etc/udev/rules.d/99-wiimote-permissions.rules`
```
DRIVERS=="wiimote", MODE="0666"
```
