import AppKit
import SwiftUI

// State

// Lightweight observable state driven entirely from the Rust side via C bridge calls.
// Recording duration is managed locally with a Swift Timer (avoids extra FFI calls).
@MainActor
final class NotchState: ObservableObject {
    static let shared = NotchState()

    // 0 = hidden, 1 = recording, 2 = transcribing
    @Published var state: Int32 = 0
    // Whether the ears are expanded (drives the width animation)
    @Published var visible: Bool = false
    @Published var audioLevel: Float = 0.0
    @Published var partialText: String = ""
    @Published var recordingDuration: TimeInterval = 0

    private var durationTimer: Timer?
    private var recordingStart: Date?

    func startRecordingTimer() {
        recordingStart = Date()
        recordingDuration = 0
        durationTimer?.invalidate()
        durationTimer = Timer.scheduledTimer(withTimeInterval: 0.1, repeats: true) { [weak self] _ in
            Task { @MainActor in
                guard let self, let start = self.recordingStart else { return }
                self.recordingDuration = Date().timeIntervalSince(start)
            }
        }
    }

    func stopRecordingTimer() {
        durationTimer?.invalidate()
        durationTimer = nil
        recordingStart = nil
    }

    func reset() {
        stopRecordingTimer()
        audioLevel = 0
        partialText = ""
        recordingDuration = 0
    }
}

// Notch Shape

struct NotchShape: Shape {
    var topCornerRadius: CGFloat
    var bottomCornerRadius: CGFloat

    init(topCornerRadius: CGFloat = 6, bottomCornerRadius: CGFloat = 14) {
        self.topCornerRadius = topCornerRadius
        self.bottomCornerRadius = bottomCornerRadius
    }

    var animatableData: AnimatablePair<CGFloat, CGFloat> {
        get { .init(topCornerRadius, bottomCornerRadius) }
        set {
            topCornerRadius = newValue.first
            bottomCornerRadius = newValue.second
        }
    }

    func path(in rect: CGRect) -> Path {
        var path = Path()
        path.move(to: CGPoint(x: rect.minX, y: rect.minY))
        path.addQuadCurve(
            to: CGPoint(x: rect.minX + topCornerRadius, y: rect.minY + topCornerRadius),
            control: CGPoint(x: rect.minX + topCornerRadius, y: rect.minY)
        )
        path.addLine(to: CGPoint(x: rect.minX + topCornerRadius, y: rect.maxY - bottomCornerRadius))
        path.addQuadCurve(
            to: CGPoint(x: rect.minX + topCornerRadius + bottomCornerRadius, y: rect.maxY),
            control: CGPoint(x: rect.minX + topCornerRadius, y: rect.maxY)
        )
        path.addLine(to: CGPoint(x: rect.maxX - topCornerRadius - bottomCornerRadius, y: rect.maxY))
        path.addQuadCurve(
            to: CGPoint(x: rect.maxX - topCornerRadius, y: rect.maxY - bottomCornerRadius),
            control: CGPoint(x: rect.maxX - topCornerRadius, y: rect.maxY)
        )
        path.addLine(to: CGPoint(x: rect.maxX - topCornerRadius, y: rect.minY + topCornerRadius))
        path.addQuadCurve(
            to: CGPoint(x: rect.maxX, y: rect.minY),
            control: CGPoint(x: rect.maxX - topCornerRadius, y: rect.minY)
        )
        path.addLine(to: CGPoint(x: rect.minX, y: rect.minY))
        return path
    }
}

// Notch Geometry

@MainActor
final class NotchGeometry: ObservableObject {
    @Published var notchWidth: CGFloat = 185
    @Published var notchHeight: CGFloat = 38
    @Published var hasNotch: Bool = false

    func update(for screen: NSScreen) {
        hasNotch = screen.safeAreaInsets.top > 0
        if hasNotch,
           let left = screen.auxiliaryTopLeftArea?.width,
           let right = screen.auxiliaryTopRightArea?.width {
            notchWidth = screen.frame.width - left - right + 4
        } else {
            notchWidth = 0
        }
        if hasNotch {
            notchHeight = screen.safeAreaInsets.top
        } else {
            let measured = screen.frame.maxY - screen.visibleFrame.maxY
            notchHeight = measured > 0 ? measured : NSStatusBar.system.thickness
        }
    }
}

// Recording Dot

struct IndicatorDot: View {
    let audioLevel: Float
    let dotPulse: Bool
    private let dotSize: CGFloat = 6

    var body: some View {
        Circle()
            .fill(Color.red)
            .frame(width: dotSize, height: dotSize)
            .scaleEffect(1.0 + CGFloat(audioLevel) * 0.8)
            .shadow(color: .yellow.opacity(dotPulse ? 0.8 : 0.2), radius: dotPulse ? 6 : 2)
    }
}

// Audio Waveform

struct AudioWaveformView: View {
    let audioLevel: Float
    let isSetup: Bool

    private let barCount = 5
    private let barWidth: CGFloat = 2.5
    private let barSpacing: CGFloat = 1.5
    private let minHeight: CGFloat = 1.5
    private let maxHeight: CGFloat = 14

    @State private var bounceIndex = 0
    @State private var bounceTimer: Timer?

    var body: some View {
        HStack(spacing: barSpacing) {
            ForEach(0..<barCount, id: \.self) { i in
                RoundedRectangle(cornerRadius: 1.5)
                    .fill(.primary)
                    .frame(width: barWidth, height: barHeight(for: i))
                    .animation(isSetup ? .easeInOut(duration: 0.3) : nil, value: bounceIndex)
            }
        }
        .frame(height: maxHeight)
        .onChange(of: isSetup) { newValue in
            if newValue { startBounce() } else { stopBounce() }
        }
        .onAppear { if isSetup { startBounce() } }
        .onDisappear { stopBounce() }
    }

    private func barHeight(for index: Int) -> CGFloat {
        isSetup ? bounceHeight(for: index) : waveformHeight(for: index)
    }

    private func waveformHeight(for index: Int) -> CGFloat {
        let level = min(Float(1.0), audioLevel * 1.4)
        let phase = Double(index) / Double(barCount) * .pi * 2
        let waveOffset = sin(phase + .pi * 0.75 + Double(level) * 3) * 0.2 + 0.8
        var barLevel = CGFloat(level) * CGFloat(waveOffset)
        if index == 0 { barLevel *= 0.8 }
        return max(minHeight, min(maxHeight, minHeight + barLevel * (maxHeight - minHeight)))
    }

    private func bounceHeight(for index: Int) -> CGFloat {
        index == bounceIndex ? 10 : minHeight
    }

    private func startBounce() {
        bounceIndex = 0
        bounceTimer?.invalidate()
        bounceTimer = Timer.scheduledTimer(withTimeInterval: 0.06, repeats: true) { _ in
            Task { @MainActor in bounceIndex = (bounceIndex + 1) % barCount }
        }
    }

    private func stopBounce() {
        bounceTimer?.invalidate()
        bounceTimer = nil
    }
}

// Expandable Text

struct IndicatorExpandableText: View {
    let text: String
    let expanded: Bool
    let contentPadding: CGFloat

    private let textFontSize: CGFloat = 12
    private let maxExpandedHeight: CGFloat = 80
    @State private var measuredTextHeight: CGFloat = 0

    var body: some View {
        ScrollViewReader { proxy in
            ScrollView(.vertical, showsIndicators: false) {
                Text(text)
                    .font(.system(size: textFontSize))
                    .foregroundStyle(.white.opacity(0.85))
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(.horizontal, contentPadding)
                    .padding(.vertical, 14)
                    .background(
                        GeometryReader { proxy in
                            Color.clear
                                .preference(
                                    key: ExpandableTextHeightPreferenceKey.self,
                                    value: proxy.size.height
                                )
                        }
                    )
                    .id("bottom")
            }
            .frame(height: expanded ? min(measuredTextHeight, maxExpandedHeight) : 0)
            .clipped()
            .onChange(of: text) { _ in
                proxy.scrollTo("bottom", anchor: .bottom)
            }
            .onPreferenceChange(ExpandableTextHeightPreferenceKey.self) { newHeight in
                guard abs(measuredTextHeight - newHeight) > 0.5 else { return }
                measuredTextHeight = newHeight
            }
        }
        .transaction { $0.disablesAnimations = true }
    }
}

private struct ExpandableTextHeightPreferenceKey: PreferenceKey {
    static var defaultValue: CGFloat = 0

    static func reduce(value: inout CGFloat, nextValue: () -> CGFloat) {
        value = nextValue()
    }
}

// Helpers

private func formatDuration(_ seconds: TimeInterval) -> String {
    let totalSeconds = Int(seconds)
    let minutes = totalSeconds / 60
    let secs = totalSeconds % 60
    return String(format: "%d:%02d", minutes, secs)
}

// Notch Indicator View

// Three-zone layout: left ear | center (notch spacer) | right ear.
// When hidden, width matches the hardware notch exactly (invisible).
// On show, ears spring outward from the notch. On hide, they retract back.
struct NotchIndicatorView: View {
    @ObservedObject private var notchState = NotchState.shared
    @ObservedObject var geometry: NotchGeometry
    @State private var dotPulse = false
    @State private var textExpanded = false

    private let extensionWidth: CGFloat = 50
    private let contentPadding: CGFloat = 16

    // Full width with both ears visible.
    private var expandedWidth: CGFloat {
        geometry.hasNotch ? geometry.notchWidth + 2 * extensionWidth : 200
    }

    // Width slightly narrower than the hardware notch so our shape sits
    // entirely inside the hardware black area without covering its rounded corners.
    private var collapsedWidth: CGFloat {
        geometry.hasNotch ? geometry.notchWidth - 20 : expandedWidth
    }

    private var hasText: Bool { textExpanded }

    private var currentWidth: CGFloat {
        if !notchState.visible {
            return collapsedWidth
        }
        if textExpanded {
            return max(expandedWidth, 340)
        }
        return expandedWidth
    }

    // Content opacity: fade in/out with the ears.
    private var contentOpacity: Double {
        notchState.visible ? 1 : 0
    }

    // On non-notch Macs, fade the entire indicator since there's no hardware notch to retract into.
    private var shapeOpacity: Double {
        geometry.hasNotch ? 1 : (notchState.visible ? 1 : 0)
    }

    // Only the bottom corners have the "ear" curves (notch Macs).
    // Non-notch Macs use straight sides (topRadius = 0).
    private var topRadius: CGFloat { geometry.hasNotch ? 8 : 0 }

    private var bottomRadius: CGFloat {
        if geometry.hasNotch {
            return hasText ? 24 : 14
        }
        return hasText ? 16 : 10
    }

    // Spring animation used for expand/collapse.
    private var expandAnimation: Animation {
        .spring(response: 0.3, dampingFraction: 1.0)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            statusBar
                .frame(width: currentWidth, height: geometry.notchHeight)
                .frame(maxWidth: .infinity)

            // Live transcription text area
            if notchState.state == 1, notchState.visible {
                IndicatorExpandableText(
                    text: notchState.partialText,
                    expanded: textExpanded,
                    contentPadding: 18
                )
                .onChange(of: notchState.partialText) { _ in
                    if !notchState.partialText.isEmpty, !textExpanded {
                        withAnimation(.easeOut(duration: 0.25)) {
                            textExpanded = true
                        }
                    }
                }
            }
        }
        .frame(width: currentWidth)
        .background(.black)
        .clipShape(NotchShape(
            topCornerRadius: topRadius,
            bottomCornerRadius: bottomRadius
        ))
        .opacity(shapeOpacity)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
        .preferredColorScheme(.dark)
        // The main expand/collapse spring drives width, corner radii, and content opacity.
        .animation(expandAnimation, value: notchState.visible)
        .animation(.easeInOut(duration: 0.3), value: textExpanded)
        .animation(.easeOut(duration: 0.08), value: notchState.audioLevel)
        .onChange(of: notchState.state) { _ in
            if notchState.state == 1 {
                withAnimation(.easeInOut(duration: 1.0).repeatForever(autoreverses: true)) {
                    dotPulse = true
                }
            } else {
                dotPulse = false
                textExpanded = false
            }
        }
        .animation(.easeInOut(duration: 1.0), value: dotPulse)
    }

    // Status bar (three-zone layout)

    @ViewBuilder
    private var statusBar: some View {
        HStack(spacing: 0) {
            // Left ear: recording dot + timer
            HStack(spacing: 4) {
                leftContent
            }
            .fixedSize()
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .leading)
            .padding(.leading, 14)
            .opacity(contentOpacity)

            // Center: transparent spacer over hardware notch
            if geometry.hasNotch {
                Color.clear.frame(width: geometry.notchWidth)
            }

            // Right ear: waveform or spinner
            rightContent
                .fixedSize()
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .trailing)
                .padding(.trailing, 16)
                .opacity(contentOpacity)
        }
    }

    @ViewBuilder
    private var leftContent: some View {
        switch notchState.state {
        case 1: // Recording
            IndicatorDot(audioLevel: notchState.audioLevel, dotPulse: dotPulse)
            Text(formatDuration(notchState.recordingDuration))
                .font(.system(size: 10, weight: .medium).monospacedDigit())
                .foregroundStyle(.primary)
        default:
            Color.clear.frame(width: 0, height: 0)
        }
    }

    @ViewBuilder
    private var rightContent: some View {
        switch notchState.state {
        case 1: // Recording — waveform
            AudioWaveformView(
                audioLevel: notchState.audioLevel,
                isSetup: notchState.recordingDuration < 0.5 && notchState.audioLevel < 0.05
            )
        case 2: // Transcribing — spinner
            ProgressView()
                .controlSize(.small)
                .tint(.primary)
        default:
            Color.clear.frame(width: 0, height: 0)
        }
    }
}

// Hosting View

private class FirstMouseHostingView<Content: View>: NSHostingView<Content> {
    override func acceptsFirstMouse(for event: NSEvent?) -> Bool { true }
}

// Notch Panel

class NotchIndicatorPanel: NSPanel {
    private static let panelWidth: CGFloat = 500
    private static let panelHeight: CGFloat = 500

    private let notchGeometry = NotchGeometry()
    private var screenObserver: NSObjectProtocol?

    init() {
        super.init(
            contentRect: NSRect(x: 0, y: 0, width: Self.panelWidth, height: Self.panelHeight),
            styleMask: [.borderless, .nonactivatingPanel, .utilityWindow, .hudWindow],
            backing: .buffered,
            defer: false
        )

        isFloatingPanel = true
        isOpaque = false
        backgroundColor = .clear
        hasShadow = false
        isMovable = false
        level = NSWindow.Level(rawValue: Int(CGShieldingWindowLevel()))
        appearance = NSAppearance(named: .darkAqua)
        collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .stationary, .ignoresCycle]
        hidesOnDeactivate = false
        ignoresMouseEvents = true
        animationBehavior = .none

        let hostingView = FirstMouseHostingView(rootView: NotchIndicatorView(geometry: notchGeometry))
        if #available(macOS 13.0, *) {
            hostingView.sizingOptions = []
        }
        contentView = hostingView

        screenObserver = NotificationCenter.default.addObserver(
            forName: NSApplication.didChangeScreenParametersNotification,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            guard let self, self.isVisible else { return }
            self.show()
        }
    }

    deinit {
        if let observer = screenObserver {
            NotificationCenter.default.removeObserver(observer)
        }
    }

    override var canBecomeKey: Bool { false }
    override var canBecomeMain: Bool { false }

    // Position the panel over the built-in display and bring to front.
    func show() {
        let screen = resolveScreen()
        notchGeometry.update(for: screen)

        let screenFrame = screen.frame
        let x = screenFrame.midX - Self.panelWidth / 2
        let y = screenFrame.origin.y + screenFrame.height - Self.panelHeight

        setFrame(NSRect(x: x, y: y, width: Self.panelWidth, height: Self.panelHeight), display: true)
        orderFrontRegardless()
    }

    private func resolveScreen() -> NSScreen {
        NSScreen.screens.first { $0.safeAreaInsets.top > 0 }
            ?? NSScreen.main
            ?? NSScreen.screens[0]
    }

    func dismiss() {
        orderOut(nil)
    }
}

// Global panel reference

private var panel: NotchIndicatorPanel?

// C Bridge Functions

private var dismissWorkItem: DispatchWorkItem?

@_cdecl("notch_indicator_init")
public func notchIndicatorInit() {
    DispatchQueue.main.async {
        panel = NotchIndicatorPanel()
    }
}

@_cdecl("notch_indicator_update_state")
public func notchIndicatorUpdateState(_ state: Int32) {
    DispatchQueue.main.async {
        let ns = NotchState.shared
        let previousState = ns.state
        ns.state = state

        // Cancel any pending dismiss so a quick re-show doesn't race.
        dismissWorkItem?.cancel()
        dismissWorkItem = nil

        let spring = Animation.spring(response: 0.3, dampingFraction: 1.0)

        switch state {
        case 0: // Hidden — retract ears, then order out after animation.
            withAnimation(spring) {
                ns.visible = false
            }
            let work = DispatchWorkItem {
                guard ns.state == 0 else { return }
                ns.reset()
                panel?.dismiss()
            }
            dismissWorkItem = work
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.35, execute: work)

        case 1: // Recording — order front (collapsed), then spring ears outward.
            if previousState != 1 {
                ns.partialText = ""
                ns.recordingDuration = 0
                ns.startRecordingTimer()
            }
            panel?.show()
            withAnimation(spring) {
                ns.visible = true
            }

        case 2: // Transcribing — keep expanded, stop timer.
            ns.stopRecordingTimer()
            panel?.show()
            withAnimation(spring) {
                ns.visible = true
            }

        default:
            withAnimation(spring) {
                ns.visible = false
            }
            let work = DispatchWorkItem {
                ns.reset()
                panel?.dismiss()
            }
            dismissWorkItem = work
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.35, execute: work)
        }
    }
}

@_cdecl("notch_indicator_update_audio_level")
public func notchIndicatorUpdateAudioLevel(_ level: Float) {
    DispatchQueue.main.async {
        NotchState.shared.audioLevel = level
    }
}

@_cdecl("notch_indicator_update_streaming_text")
public func notchIndicatorUpdateStreamingText(_ text: UnsafePointer<CChar>?) {
    guard let text else { return }
    let swiftText = String(cString: text)
    DispatchQueue.main.async {
        NotchState.shared.partialText = swiftText
    }
}

@_cdecl("notch_indicator_destroy")
public func notchIndicatorDestroy() {
    DispatchQueue.main.async {
        NotchState.shared.visible = false
        NotchState.shared.reset()
        panel?.dismiss()
        panel = nil
    }
}
