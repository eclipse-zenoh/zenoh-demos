package com.example.zenohapp.ui.examples

import io.zenoh.keyexpr.intoKeyExpr
import io.zenoh.liveliness.LivelinessToken

class ZLivelinessFragment : ZExampleFragment() {

    private var token: LivelinessToken? = null

    override fun startExample() {
        val session = viewModel.zenohSession!!
        val keyExpr = "group1/zenoh-kotlin-android".intoKeyExpr().getOrThrow()
        token = session.liveliness().declareToken(keyExpr).onSuccess {
            writeToConsole("Declared liveliness token on '$keyExpr'...")
        }.getOrThrow()
    }

    override fun stopExample() {
        token?.close()
    }
}