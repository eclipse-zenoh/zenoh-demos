package com.example.zenohdemo.ui.examples

import com.example.zenohdemo.R
import io.zenoh.keyexpr.intoKeyExpr
import io.zenoh.pubsub.Subscriber
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

class ZSubThrFragment : ZExampleFragment() {

    companion object {
        private const val NANOS_TO_SEC = 1_000_000_000L
    }

    private var batchCount = 0u
    private var count = 0u
    private var startTimestampNs: Long = 0
    private var globalStartTimestampNs: Long = 0
    private val number = 100000u
    private val samples = 10u

    private var subscriber: Subscriber<Unit>? = null

    private fun listener() {
        if (batchCount > samples) {
            report()
            stopExample()
        }
        if (count == 0u) {
            startTimestampNs = System.nanoTime()
            if (globalStartTimestampNs == 0L) {
                globalStartTimestampNs = startTimestampNs
            }
            count++
            return
        }
        if (count < number) {
            count++
            return
        }
        val stop = System.nanoTime()
        val elapsedTimeSecs = (stop - startTimestampNs).toDouble() / NANOS_TO_SEC
        val messagesPerSec = number.toLong() / elapsedTimeSecs
        writeToConsole("%.2f msgs/sec".format(messagesPerSec))
        batchCount++
        count = 0u
    }

    private fun report() {
        val end = System.nanoTime()
        val total = batchCount * number + count
        val elapsedTimeSecs = (end - globalStartTimestampNs).toDouble() / NANOS_TO_SEC
        val globalMessagesPerSec = total.toLong() / elapsedTimeSecs
        writeToConsole(
            "Received $total messages in %.2f seconds: averaged %.2f msgs/sec".format(
                elapsedTimeSecs,
                globalMessagesPerSec
            )
        )
    }

    override fun startExample() {
        val session = viewModel.zenohSession!!
        val keyExpr = "test/thr".intoKeyExpr().getOrThrow()
        subscriber =
            session.declareSubscriber(
                keyExpr,
                callback = { listener() },
            ).getOrThrow()

        writeToConsole("Started ZSubThr on '$keyExpr'...")
    }

    @OptIn(DelicateCoroutinesApi::class)
    override fun stopExample() {
        subscriber?.undeclare()
        exampleIsRunning = false
        batchCount = 0u
        count = 0u
        GlobalScope.launch {
            withContext(Dispatchers.Main) {
                button.text = getString(R.string.start)
            }
        }
    }

    override fun onDestroyView() {
        exampleIsRunning = false
        subscriber?.undeclare()
        super.onDestroyView()
    }
}
