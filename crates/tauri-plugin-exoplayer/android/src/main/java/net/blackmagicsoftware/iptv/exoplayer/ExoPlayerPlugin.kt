package net.blackmagicsoftware.iptv.exoplayer

import android.app.Activity
import android.net.Uri
import android.util.Log
import android.view.Gravity
import android.view.View
import android.view.ViewGroup
import android.widget.FrameLayout
import androidx.annotation.OptIn
import androidx.media3.common.MediaItem
import androidx.media3.common.MimeTypes
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
import androidx.media3.common.util.UnstableApi
import androidx.media3.datasource.DefaultHttpDataSource
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.source.DefaultMediaSourceFactory
import androidx.media3.exoplayer.source.UnrecognizedInputFormatException
import androidx.media3.ui.PlayerView
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

@InvokeArg
class LoadArgs {
    lateinit var url: String
}

@InvokeArg
class VolumeArgs {
    var level: Double = 100.0
}

@InvokeArg
class MuteArgs {
    var muted: Boolean = false
}

@TauriPlugin
class ExoPlayerPlugin(private val activity: Activity) : Plugin(activity) {
    private var player: ExoPlayer? = null
    private var playerView: PlayerView? = null
    private var container: FrameLayout? = null
    private var lastError: String? = null
    private var activeUrl: String? = null
    private var mimeFallbacks: List<String?> = emptyList()
    private var mimeFallbackIndex = 0
    private var lastLoadedHost: String? = null

    companion object {
        private const val TAG = "ExoPlayerPlugin"
        private const val USER_AGENT = "VLC/3.0.21 LibVLC/3.0.21"
    }

    private fun ensurePlayer() {
        if (player != null) return

        val dataSourceFactory = DefaultHttpDataSource.Factory()
            .setUserAgent(USER_AGENT)
            .setAllowCrossProtocolRedirects(true)
            .setConnectTimeoutMs(15_000)
            .setReadTimeoutMs(30_000)

        val mediaSourceFactory = DefaultMediaSourceFactory(activity)
            .setDataSourceFactory(dataSourceFactory)

        val exoPlayer = ExoPlayer.Builder(activity)
            .setMediaSourceFactory(mediaSourceFactory)
            .build()

        exoPlayer.addListener(object : Player.Listener {
            override fun onPlayerError(error: PlaybackException) {
                if (tryNextMimeFallback()) {
                    return
                }
                lastError = formatPlaybackError(error)
                Log.e(TAG, "Playback error: $lastError", error)
            }

            override fun onPlaybackStateChanged(playbackState: Int) {
                if (playbackState == Player.STATE_READY) {
                    lastError = null
                }
            }
        })

        player = exoPlayer

        val frame = FrameLayout(activity).apply {
            layoutParams = FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.MATCH_PARENT,
                Gravity.CENTER
            )
            visibility = View.GONE
            setBackgroundColor(0xFF000000.toInt())
        }
        container = frame

        val view = PlayerView(activity).apply {
            layoutParams = FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.MATCH_PARENT
            )
            useController = false
            this.player = exoPlayer
        }
        playerView = view
        frame.addView(view)

        // Behind the WebView so Svelte controls stay on top (requires transparent WebView).
        val content = activity.findViewById<ViewGroup>(android.R.id.content)
        content.addView(frame, 0, frame.layoutParams)
    }

    private fun formatPlaybackError(error: PlaybackException): String {
        val cause = generateSequence<Throwable>(error) { it.cause }.lastOrNull() ?: error
        return when (cause) {
            is UnrecognizedInputFormatException ->
                "Stream format not recognized. The provider may require a different URL type (HLS vs MPEG-TS), or returned a non-video response."
            else -> cause.message ?: error.message ?: "playback error"
        }
    }

    private fun primaryMimeForUrl(url: String): String? {
        val lower = url.lowercase()
        val path = Uri.parse(url).path?.lowercase() ?: lower

        return when {
            lower.contains(".m3u8") ||
                lower.contains("type=m3u8") ||
                lower.contains("output=m3u8") ||
                lower.contains("type=hls") ||
                lower.contains("output=hls") -> MimeTypes.APPLICATION_M3U8
            path.endsWith(".ts") ||
                lower.contains("type=ts") ||
                lower.contains(".ts?") -> MimeTypes.VIDEO_MP2T
            path.endsWith(".mpd") -> MimeTypes.APPLICATION_MPD
            path.endsWith(".mp4") -> MimeTypes.VIDEO_MP4
            path.endsWith(".mkv") -> MimeTypes.VIDEO_MATROSKA
            else -> null
        }
    }

    private fun mimeCandidatesForUrl(url: String): List<String?> {
        val primary = primaryMimeForUrl(url)
        val candidates = mutableListOf<String?>()
        if (primary != null) {
            candidates.add(primary)
        } else {
            // Extensionless IPTV URLs are usually MPEG-TS, not HLS manifests.
            candidates.add(MimeTypes.VIDEO_MP2T)
            candidates.add(MimeTypes.APPLICATION_M3U8)
            candidates.add(null)
            return candidates
        }
        if (primary != MimeTypes.APPLICATION_M3U8) {
            candidates.add(MimeTypes.APPLICATION_M3U8)
        }
        if (primary != MimeTypes.VIDEO_MP2T) {
            candidates.add(MimeTypes.VIDEO_MP2T)
        }
        candidates.add(null)
        return candidates.distinct()
    }

    private fun buildMediaItem(url: String, mime: String?): MediaItem {
        val builder = MediaItem.Builder().setUri(Uri.parse(url))
        if (mime != null) {
            builder.setMimeType(mime)
        }
        return builder.build()
    }

    private fun startLoad(url: String) {
        activeUrl = url
        mimeFallbacks = mimeCandidatesForUrl(url)
        mimeFallbackIndex = 0
        lastLoadedHost = Uri.parse(url).host
        val mime = mimeFallbacks[mimeFallbackIndex]
        Log.i(TAG, "Loading host=$lastLoadedHost mime=$mime")

        val exo = player ?: return
        exo.stop()
        exo.clearMediaItems()
        exo.setMediaItem(buildMediaItem(url, mime))
        exo.prepare()
    }

    private fun tryNextMimeFallback(): Boolean {
        val url = activeUrl ?: return false
        if (mimeFallbackIndex + 1 >= mimeFallbacks.size) {
            return false
        }
        mimeFallbackIndex++
        val mime = mimeFallbacks[mimeFallbackIndex]
        Log.i(TAG, "Retrying host=$lastLoadedHost mime=$mime (attempt ${mimeFallbackIndex + 1})")

        val exo = player ?: return false
        val shouldPlay = exo.playWhenReady
        exo.stop()
        exo.clearMediaItems()
        exo.setMediaItem(buildMediaItem(url, mime))
        exo.prepare()
        exo.playWhenReady = shouldPlay
        return true
    }

    @OptIn(UnstableApi::class)
    @Command
    fun load(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(LoadArgs::class.java)
            ensurePlayer()
            if (player == null) {
                invoke.reject("player not initialized")
                return
            }
            lastError = null
            startLoad(args.url)
            invoke.resolve()
        } catch (e: Exception) {
            invoke.reject(e.message ?: "load failed")
        }
    }

    @Command
    fun play(invoke: Invoke) {
        try {
            player?.playWhenReady = true
            invoke.resolve()
        } catch (e: Exception) {
            invoke.reject(e.message ?: "play failed")
        }
    }

    @Command
    fun pause(invoke: Invoke) {
        try {
            player?.playWhenReady = false
            invoke.resolve()
        } catch (e: Exception) {
            invoke.reject(e.message ?: "pause failed")
        }
    }

    @Command
    fun stop(invoke: Invoke) {
        try {
            player?.stop()
            activeUrl = null
            invoke.resolve()
        } catch (e: Exception) {
            invoke.reject(e.message ?: "stop failed")
        }
    }

    @Command
    fun setVolume(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(VolumeArgs::class.java)
            val vol = (args.level / 100.0).toFloat().coerceIn(0f, 1f)
            player?.volume = vol
            invoke.resolve()
        } catch (e: Exception) {
            invoke.reject(e.message ?: "setVolume failed")
        }
    }

    @Command
    fun setMuted(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(MuteArgs::class.java)
            player?.volume = if (args.muted) 0f else 1f
            invoke.resolve()
        } catch (e: Exception) {
            invoke.reject(e.message ?: "setMuted failed")
        }
    }

    @Command
    fun showPlayer(invoke: Invoke) {
        try {
            container?.visibility = View.VISIBLE
            invoke.resolve()
        } catch (e: Exception) {
            invoke.reject(e.message ?: "showPlayer failed")
        }
    }

    @Command
    fun hidePlayer(invoke: Invoke) {
        try {
            container?.visibility = View.GONE
            invoke.resolve()
        } catch (e: Exception) {
            invoke.reject(e.message ?: "hidePlayer failed")
        }
    }

    @Command
    fun getStats(invoke: Invoke) {
        try {
            val exo = player
            val ret = JSObject()
            if (exo == null) {
                invoke.resolve(ret)
                return
            }
            lastError?.let { ret.put("error", it) }
            lastLoadedHost?.let { ret.put("host", it) }
            val format = exo.videoFormat
            if (format != null) {
                if (format.width > 0) ret.put("width", format.width)
                if (format.height > 0) ret.put("height", format.height)
                format.codecs?.let { ret.put("videoCodec", it) }
            }
            exo.audioFormat?.codecs?.let { ret.put("audioCodec", it) }
            invoke.resolve(ret)
        } catch (e: Exception) {
            invoke.reject(e.message ?: "getStats failed")
        }
    }
}
