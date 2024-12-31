package com.example.zenohapp.ui.examples

class ZInfoFragment : ZExampleFragment() {

    override fun startExample() {
        val session = viewModel.zenohSession!!
        val info = session.info()

        writeToConsole("zid: ${info.zid().getOrThrow()}")
        writeToConsole("routers zid: ${info.routersZid().getOrThrow()}")
        writeToConsole("peers zid: ${info.peersZid().getOrThrow()}")
    }

    override fun stopExample() {}
}