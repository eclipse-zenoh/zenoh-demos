package com.example.zenohdemo.ui.examples

import io.zenoh.bytes.ZBytes
import io.zenoh.keyexpr.intoKeyExpr

class ZPutFragment : ZExampleFragment() {
    companion object {
        private val TAG = ZPutFragment::javaClass.name
    }

    override fun startExample() {
        val session = viewModel.zenohSession!!
        val keyExpr = "demo/example/zenoh-android-put".intoKeyExpr().getOrThrow()
        val payload = ZBytes.from("Put from Android!")
        session.put(keyExpr, payload).onSuccess {
            writeToConsole("Putting Data ('$keyExpr': '$payload')...")
            resetState()
        }.onFailure { handleError(TAG, "Failed to perform PUT", it) }
    }

    override fun stopExample() {}
}