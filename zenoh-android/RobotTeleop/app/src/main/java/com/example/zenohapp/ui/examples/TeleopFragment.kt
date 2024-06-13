package com.example.zenohapp.ui.examples

import android.os.Bundle
import android.util.Log
import android.view.LayoutInflater
import android.view.MotionEvent
import android.view.View
import android.view.View.OnClickListener
import android.view.View.OnTouchListener
import android.view.ViewGroup
import android.widget.Button
import android.widget.EditText
import android.widget.ImageButton
import android.widget.ImageView
import android.widget.ProgressBar
import android.widget.SeekBar
import android.widget.SeekBar.OnSeekBarChangeListener
import android.widget.Toast
import androidx.fragment.app.Fragment
import androidx.lifecycle.ViewModelProvider
import com.example.zenohapp.R
import com.example.zenohapp.ZenohViewModel
import com.example.zenohapp.cdr.CDRInputStream
import com.example.zenohapp.cdr.CDROutputStream
import com.example.zenohapp.databinding.FragmentTeleopBinding
import com.example.zenohapp.ros.AudioNote
import com.example.zenohapp.ros.AudioNoteVector
import com.example.zenohapp.ros.Battery
import com.example.zenohapp.ros.Header
import com.example.zenohapp.ros.Image
import com.example.zenohapp.ros.Time
import com.example.zenohapp.ros.Twist
import com.example.zenohapp.ros.Vector3
import io.zenoh.keyexpr.intoKeyExpr
import io.zenoh.prelude.Encoding
import io.zenoh.prelude.KnownEncoding
import io.zenoh.publication.Publisher
import io.zenoh.sample.Sample
import io.zenoh.subscriber.Subscriber
import io.zenoh.value.Value
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancel
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import java.nio.ByteBuffer
import kotlin.random.Random





class TeleopFragment : Fragment(), OnTouchListener, OnClickListener, OnSeekBarChangeListener {
    private var _binding: FragmentTeleopBinding? = null
    private val binding get() = _binding!!


    private lateinit var mFwdButton: ImageButton
    private lateinit var mBwdButton: ImageButton
    private lateinit var mLeftButton: ImageButton
    private lateinit var mRightButton: ImageButton
    private lateinit var mHomeButton: ImageButton
    private lateinit var mSoundButton : ImageButton
    private lateinit var mDeclareButton: Button
    private lateinit var mNSEditText : EditText
    private lateinit var mLinearSeekBar: SeekBar
    private lateinit var mAngularSeekBar: SeekBar
    private lateinit var mCameraView: ImageView
    private lateinit var mBatteryBar : ProgressBar
    private lateinit var viewModel: ZenohViewModel

    private lateinit var mKeyExpr: String
    private lateinit var mPublisher: Publisher
    private lateinit var mBatterySubscriber: Subscriber<Channel<Sample>>
    private lateinit var mTwist: Twist
    private var isSending = false
    private lateinit var mBatterySubJob : Job
    private lateinit var mCameraSubJob : Job
    private lateinit var mCommandPubJob : Job

    private val actionDockSuffix = "/dock/_action/send_goal"
    private val cmdVelSuffix = "/cmd_vel"
    private val batteryStateSuffix = "/battery_state"
    private val imageRawSuffix = "/oakd/rgb/preview/image_raw"
    private var angularScale = 0.5
    private var linearScale = 0.5
    override fun onTouch(v: View?, event: MotionEvent?): Boolean {
        when (v?.id) {
            R.id.buttonFwd -> {

                when (event?.action) {
                    MotionEvent.ACTION_DOWN -> {
                        isSending = true
                        setCmd(MovementDirection.FWD)
                    }

                    MotionEvent.ACTION_UP -> {
                        setCmd(MovementDirection.STOP)
                    }

                    else -> {}
                }

            }

            R.id.buttonBwd -> {
                when (event?.action) {
                    MotionEvent.ACTION_DOWN -> {
                        isSending = true
                        setCmd(MovementDirection.BWD)
                    }

                    MotionEvent.ACTION_UP -> {
                        setCmd(MovementDirection.STOP)
                    }

                    else -> {}
                }
            }

            R.id.buttonLeft -> {
                when (event?.action) {
                    MotionEvent.ACTION_DOWN -> {
                        isSending = true
                        setCmd(MovementDirection.LEFT)
                    }

                    MotionEvent.ACTION_UP -> {
                        setCmd(MovementDirection.STOP)
                    }

                    else -> {}
                }
            }

            R.id.buttonRight -> {
                when (event?.action) {
                    MotionEvent.ACTION_DOWN -> {
                        isSending = true
                        setCmd(MovementDirection.RIGHT)
                    }

                    MotionEvent.ACTION_UP -> {
                        setCmd(MovementDirection.STOP)
                    }

                    else -> {}
                }
            }

            else -> {}
        }
        return true
    }

    companion object {
        private val TAG = TeleopFragment::javaClass.name
    }


    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

    }

    override fun onCreateView(
        inflater: LayoutInflater, container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        viewModel = ViewModelProvider(requireActivity()).get(ZenohViewModel::class.java)
        _binding = FragmentTeleopBinding.inflate(inflater, container, false)
        // Inflate the layout for this fragment
        val root: View = binding.root

        mTwist = Twist(Vector3(0.0, 0.0, 0.0), Vector3(0.0, 0.0, 0.0))

        mFwdButton = binding.buttonFwd
        mBwdButton = binding.buttonBwd
        mLeftButton = binding.buttonLeft
        mRightButton = binding.buttonRight
        mHomeButton = binding.goHomeButton
        mSoundButton = binding.soundButton
        mNSEditText = binding.editTextNamespace
        mLinearSeekBar = binding.seekLinearSensitivity
        mAngularSeekBar = binding.seekAngularSensitivity
        mDeclareButton = binding.buttonDeclare
        mCameraView = binding.imageViewCamera
        mBatteryBar = binding.robotBatteryBar

//        mCameraView.visibility = View.INVISIBLE


        mDeclareButton.setOnClickListener(this)
        mAngularSeekBar.setOnSeekBarChangeListener(this)
        mLinearSeekBar.setOnSeekBarChangeListener(this)

        mFwdButton.setOnTouchListener(this)
        mBwdButton.setOnTouchListener(this)
        mHomeButton.setOnClickListener(this)
        mSoundButton.setOnClickListener(this)
        mLeftButton.setOnTouchListener(this)
        mRightButton.setOnTouchListener(this)


        setScale()
        declarePublisher()
        startPubSub()

        return root
    }

    private fun declarePublisher() {
        mKeyExpr = mNSEditText.text.toString()+cmdVelSuffix
        viewModel.zenohSession?.apply {
            mKeyExpr.intoKeyExpr().onSuccess { ke ->
                this.declarePublisher(ke).res().onSuccess { pub ->
                    mPublisher = pub
                    Log.v(TAG, "Declared publiser on: ${mNSEditText.text.toString()}")
                }
                    .onFailure { handleError(TAG, "Failed to launch publisher", it) }
            }
                .onFailure { handleError(TAG, "Failed to parse keyExpr", it) }
        }
    }


    private fun publishCommand(cmd: Twist) {

        var mStream = CDROutputStream()
        cmd.serialize(mStream)
        var arr = ByteArray(mStream.buffer.position())
        mStream.buffer.rewind()
        mStream.buffer.get(arr)
        val payload = Value(arr, Encoding(KnownEncoding.APP_OCTET_STREAM))

        mPublisher.put(payload).res().onFailure { Log.e(TAG, "Error when publishing", it) }
            .onSuccess { Log.v(TAG, "Published command!") }
    }

    private fun setCmd(direction: MovementDirection) {
        when (direction) {
            MovementDirection.RIGHT -> {
                mTwist.angular.z = (-1.0 * angularScale)
            }

            MovementDirection.LEFT -> {
                mTwist.angular.z = (1.0 * angularScale)
            }

            MovementDirection.FWD -> {
                mTwist.linear.x = (1.0 * linearScale)
            }

            MovementDirection.BWD -> {
                mTwist.linear.x = (-1.0 * linearScale)
            }

            MovementDirection.STOP -> {
                mTwist.linear.x = 0.0
                mTwist.angular.z = 0.0
            }
        }
        //publishCommand(mTwist)

    }

    private fun handleError(tag: String, errorMsg: String, error: Throwable) {
        Log.e(tag, "$errorMsg: $error")
        Toast.makeText(
            activity,
            errorMsg,
            Toast.LENGTH_SHORT
        ).show()
    }

    override fun onClick(v: View?) {
        when (v?.id) {
            R.id.buttonDeclare -> {
                declarePublisher()
                startPubSub()
            }

            R.id.goHomeButton -> {
                sendHomeCmd()
            }
            R.id.soundButton -> {
                sendSoundCmd()
            }

            else -> {}
        }
    }

    override fun onProgressChanged(seekBar: SeekBar?, progress: Int, fromUser: Boolean) {
        when (seekBar?.id) {
            R.id.seekLinearSensitivity -> {
                setScale()
            }

            R.id.seekAngularSensitivity -> {
                setScale()
            }

            else -> {}

        }
    }

    private fun sendHomeCmd() {
        val actionDockKE = mNSEditText.text.toString() + actionDockSuffix
        var mStream = CDROutputStream()
        // Request needs a random Uuid, no other fields
        for (i in 0..16) {
            mStream.writeByte(Random.nextBits(8).toByte())
        }

        var arr = ByteArray(mStream.buffer.position())
        mStream.buffer.rewind()
        mStream.buffer.get(arr)
        val payload = Value(arr, Encoding(KnownEncoding.APP_OCTET_STREAM))

        viewModel.zenohSession?.apply {
            actionDockKE.intoKeyExpr().onSuccess { ke ->
                this.get(ke)
                    .withValue(payload)
                    .res().onSuccess {

                        Log.v(TAG, "Sent dock action publiser : $actionDockKE")
                    }
                    .onFailure { handleError(TAG, "Failed to send action dock", it) }
            }
                .onFailure { handleError(TAG, "Failed to parse keyExpr", it) }
        }
    }

    private fun sendSoundCmd() {
        var mStream = CDROutputStream()

        val note1 = AudioNote(369u, Time(0, 355000000u))
        val note2 = AudioNote(300u, Time(0, 533000000u))
        var notes = arrayOf(note1, note2)

        val sound = AudioNoteVector(
            Header(Time(0,0u), ""),
            notes,
            false,
        )

        sound.serialize(mStream)

        var arr = ByteArray(mStream.buffer.position())
        mStream.buffer.rewind()
        mStream.buffer.get(arr)
        val payload = Value(arr, Encoding(KnownEncoding.APP_OCTET_STREAM))

        viewModel.zenohSession?.apply {
            "zbot_2/cmd_audio".intoKeyExpr().onSuccess { ke ->
                this.put(ke, payload)
                    .res().onSuccess {
                        Log.v(TAG, "Sent sound command")
                    }
                    .onFailure { handleError(TAG, "Failed to send action dock", it) }
            }
                .onFailure { handleError(TAG, "Failed to parse keyExpr", it) }
        }

    }

    private fun startPubSub() {
        if (::mBatterySubJob.isInitialized) {
            mBatterySubJob?.cancel()
        }
        if (::mBatterySubJob.isInitialized) {
            mBatterySubJob?.cancel()
        }
        if (::mCommandPubJob.isInitialized) {
            mCommandPubJob?.cancel()
        }


        // Subscriber battery coorutine
        viewModel.zenohSession?.apply {
            (mNSEditText.text.toString() + batteryStateSuffix).intoKeyExpr().onSuccess { key ->
                Log.v(TAG, "Subscribing battery state on: $key")
                this.declareSubscriber(key).res().onSuccess { sub ->
                    mBatterySubscriber = sub
                    mBatterySubJob =  GlobalScope.launch(Dispatchers.IO) {
                        sub.receiver?.apply {
                            val iterator = this.iterator()
                            while (iterator.hasNext()) {
                                val sample = iterator.next()
                                //Log.v(TAG, "Received data from Zenoh")
                                val inputStream =
                                    CDRInputStream(ByteBuffer.wrap(sample.value.payload))
                                //Log.v(TAG, "Size from Zenoh is: ${inputStream.buffer.capacity()} - Position: ${inputStream.buffer.position()}")
                                val battery = Battery(inputStream)
                                Log.v(TAG, "Battery percentage: ${battery.percentage}")
                                mBatteryBar.progress = (battery.percentage * 100.0f).toInt()
                            }
                        }
                    }
                }.onFailure { Log.w(TAG, "Unable to subscribe to battery") }
            }.onFailure { Log.w(TAG, "Unable to get session") }
        }

        // Publish coorutine
        mCommandPubJob = GlobalScope.launch(Dispatchers.IO) {
            while (true) {
                var zCount = 0
                while (isSending) {
                    delay(100)
                    publishCommand(mTwist)
                    if (mTwist.isStopped()) {
                        zCount += 1
                    }
                    if (zCount == 10) {
                        isSending = false
                    }

                }
                delay(100)
            }
        }

        // Subscribe camera coorutine
//        viewModel.zenohSession?.apply {
//            (mNSEditText.text.toString() + imageRawSuffix).intoKeyExpr().onSuccess { key ->0-
//                Log.v(TAG, "Subscribing battery state on: $key")
//                this.declareSubscriber(key).res().onSuccess { sub ->
//                    mBatterySubscriber = sub
//                        mCameraSubJob = GlobalScope.launch(Dispatchers.Main) {
//                        sub.receiver?.apply {
//                            val iterator = this.iterator()
//                            while (iterator.hasNext()) {
//                                val sample = iterator.next()
//                                //Log.v(TAG, "Received data from Zenoh")
//                                val inputStream =
//                                    CDRInputStream(ByteBuffer.wrap(sample.value.payload))
//                                //Log.v(TAG, "Size from Zenoh is: ${inputStream.buffer.capacity()} - Position: ${inputStream.buffer.position()}")
//                                val image = Image(inputStream)
//                                Log.v(TAG, "Image is ${image.height}x${image.width} format: ${image.encoding}")
//
//                                val bmp = image.toBitmap()
//                                mCameraView.setImageBitmap(bmp)
//
//
//
//                            }
//                        }
//                    }
//                }.onFailure { Log.w(TAG, "Unable to subscribe to battery") }
//            }.onFailure { Log.w(TAG, "Unable to get session") }
//        }
    }

    private fun setScale() {
        linearScale = mLinearSeekBar.progress.toDouble() / 100.0
        angularScale = mAngularSeekBar.progress.toDouble() / 100.0
        Log.v(TAG, "Linear Scale: $linearScale - Angular Scale: $angularScale")
    }

    override fun onStartTrackingTouch(seekBar: SeekBar?) {
        return
    }

    override fun onStopTrackingTouch(seekBar: SeekBar?) {
        return
    }

    enum class MovementDirection {
        FWD, BWD, LEFT, RIGHT, STOP
    }
}