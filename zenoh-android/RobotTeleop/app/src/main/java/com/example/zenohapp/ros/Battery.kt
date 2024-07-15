package com.example.zenohapp.ros

import android.util.Log
import com.example.zenohapp.cdr.CDRInputStream
import com.example.zenohapp.ros.Header
import java.lang.reflect.Constructor

class Battery constructor(
    var header: Header,
    var voltage: Float,
    val temperature: Float,
    var current: Float,
    var charge: Float,
    var capacity: Float,
    var designCapacity: Float,
    var percentage: Float,
    var powerSupplyStatus: UByte,
    var powerSupplyHealth: UByte,
    var powerSupplyTechnology: UByte,
    var present: Boolean,
    var cellVoltage: Array<Float>,
    var cellTemperature: Array<Float>,
    var location: String,
    var serialNumber: String
) {
    constructor (inputStream: CDRInputStream) : this(
        Header(inputStream),
        inputStream.readFloat(),
        inputStream.readFloat(),
        inputStream.readFloat(),
        inputStream.readFloat(),
        inputStream.readFloat(),
        inputStream.readFloat(),
        inputStream.readFloat(),
        inputStream.readUByte(),
        inputStream.readUByte(),
        inputStream.readUByte(),
        inputStream.readBoolean(),
        inputStream.readFloatArray(),
        inputStream.readFloatArray(),
        inputStream.readString(),
        inputStream.readString(),
    ) {
        Log.v("BatteryConstructor", "streamPos: ${inputStream.buffer.position()} - ${this.toString()}")
    }


    override fun toString(): String {
        return "Header: ${header.toString()} " +
                "- voltage: $voltage " +
                "- current: $current " +
                "- charge: $charge " +
                "- capacity: $capacity " +
                "- design_capacity: $designCapacity " +
                "- percentage: $percentage " +
                "- power_supply_status: $powerSupplyStatus " +
                "- power_supply_health: $powerSupplyHealth" +
                "- power_supply_technology: $powerSupplyTechnology " +
                "- present: $present " +
                "- location: $location " +
                "- sn: $serialNumber"

    }
}

