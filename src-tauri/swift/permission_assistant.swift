import AppKit
import CoreGraphics
import Foundation

private enum PermissionPanel: Int32 {
    case accessibility = 0
    case microphone = 1

    var title: String {
        switch self {
        case .accessibility:
            return "Enable Accessibility Access"
        case .microphone:
            return "Enable Microphone Access"
        }
    }

    var message: String {
        switch self {
        case .accessibility:
            return "Drag Handless into the Accessibility list, then switch it on so the app can type transcribed text for you."
        case .microphone:
            return "Turn on Handless in the Microphone list so the app can capture your voice."
        }
    }

    var settingsURL: URL {
        let identifier: String
        switch self {
        case .accessibility:
            identifier = "Privacy_Accessibility"
        case .microphone:
            identifier = "Privacy_Microphone"
        }

        guard let url = URL(
            string: "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension?\(identifier)"
        ) else {
            preconditionFailure("Invalid System Settings URL for \(identifier)")
        }
        return url
    }

    var showsDragSource: Bool {
        self == .accessibility
    }
}

private struct HostApp {
    let displayName: String
    let bundleURL: URL
    let icon: NSImage

    static func current(bundle: Bundle = .main) -> HostApp {
        let bundleURL = bundle.bundleURL.resolvingSymlinksInPath()
        if bundleURL.pathExtension == "app", let hostApp = from(bundleURL: bundleURL) {
            return hostApp
        }

        if let hostApp = ancestorApp() {
            return hostApp
        }

        if let executableURL = bundle.executableURL?.resolvingSymlinksInPath() {
            let targetURL = executableURL.deletingLastPathComponent()
            let icon = NSWorkspace.shared.icon(forFile: targetURL.path)
            icon.size = NSSize(width: 48, height: 48)
            return HostApp(
                displayName: targetURL.lastPathComponent,
                bundleURL: targetURL,
                icon: icon
            )
        }

        let displayName =
            bundle.object(forInfoDictionaryKey: "CFBundleDisplayName") as? String
            ?? bundle.object(forInfoDictionaryKey: kCFBundleNameKey as String) as? String
            ?? bundleURL.deletingPathExtension().lastPathComponent
        let icon = NSWorkspace.shared.icon(forFile: bundleURL.path)
        icon.size = NSSize(width: 48, height: 48)
        return HostApp(displayName: displayName, bundleURL: bundleURL, icon: icon)
    }

    private static func from(bundleURL: URL) -> HostApp? {
        guard let bundle = Bundle(url: bundleURL) else {
            return nil
        }

        let displayName =
            bundle.object(forInfoDictionaryKey: "CFBundleDisplayName") as? String
            ?? bundle.object(forInfoDictionaryKey: kCFBundleNameKey as String) as? String
            ?? bundleURL.deletingPathExtension().lastPathComponent
        let icon = NSWorkspace.shared.icon(forFile: bundleURL.path)
        icon.size = NSSize(width: 48, height: 48)
        return HostApp(displayName: displayName, bundleURL: bundleURL, icon: icon)
    }

    private static func ancestorApp(startingPID: pid_t = getppid()) -> HostApp? {
        var pid = startingPID

        while pid > 1 {
            if let bundleURL = bundleURL(for: pid),
               let hostApp = from(bundleURL: bundleURL)
            {
                return hostApp
            }

            pid = parentPID(for: pid)
        }

        return nil
    }

    private static func bundleURL(for pid: pid_t) -> URL? {
        guard let runningApp = NSRunningApplication(processIdentifier: pid) else {
            return nil
        }

        return runningApp.bundleURL
    }

    private static func parentPID(for pid: pid_t) -> pid_t {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/bin/ps")
        process.arguments = ["-p", String(pid), "-o", "ppid="]

        let pipe = Pipe()
        process.standardOutput = pipe
        process.standardError = Pipe()

        do {
            try process.run()
        } catch {
            return 0
        }

        process.waitUntilExit()
        guard process.terminationStatus == 0 else {
            return 0
        }

        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        guard
            let output = String(data: data, encoding: .utf8)?
                .trimmingCharacters(in: .whitespacesAndNewlines),
            let parent = Int32(output)
        else {
            return 0
        }

        return pid_t(parent)
    }
}

private struct SettingsWindowSnapshot: Equatable {
    let frame: CGRect
    let visibleFrame: CGRect
}

private enum SettingsWindowLocator {
    static let bundleIdentifier = "com.apple.systempreferences"

    static var isSystemSettingsFrontmost: Bool {
        NSWorkspace.shared.frontmostApplication?.bundleIdentifier == bundleIdentifier
    }

    static func frontmostWindow() -> SettingsWindowSnapshot? {
        guard isSystemSettingsFrontmost else {
            return nil
        }

        guard
            let app = NSRunningApplication.runningApplications(
                withBundleIdentifier: bundleIdentifier
            ).max(by: {
                ($0.activationPolicy == .prohibited ? 0 : 1)
                    < ($1.activationPolicy == .prohibited ? 0 : 1)
            })
        else {
            return nil
        }

        guard
            let windowInfo = CGWindowListCopyWindowInfo(
                [.optionOnScreenOnly, .excludeDesktopElements],
                .zero
            ) as? [[String: Any]]
        else {
            return nil
        }

        let windows = windowInfo.compactMap { info -> SettingsWindowSnapshot? in
            guard
                let ownerPID = info[kCGWindowOwnerPID as String] as? pid_t,
                ownerPID == app.processIdentifier
            else {
                return nil
            }
            guard let layer = info[kCGWindowLayer as String] as? Int, layer == 0 else {
                return nil
            }
            guard let bounds = info[kCGWindowBounds as String] as? [String: CGFloat] else {
                return nil
            }

            let cgFrame = CGRect(
                x: bounds["X"] ?? 0,
                y: bounds["Y"] ?? 0,
                width: bounds["Width"] ?? 0,
                height: bounds["Height"] ?? 0
            )
            let converted = appKitGeometry(from: cgFrame)
            let frame = converted.frame
            guard frame.width > 320, frame.height > 240 else {
                return nil
            }

            return SettingsWindowSnapshot(
                frame: frame,
                visibleFrame: converted.visibleFrame
            )
        }

        return windows.max(by: {
            $0.frame.width * $0.frame.height < $1.frame.width * $1.frame.height
        })
    }

    private static func appKitGeometry(from cgFrame: CGRect) -> (frame: CGRect, visibleFrame: CGRect) {
        let screens = NSScreen.screens.compactMap { screen -> (frame: CGRect, visibleFrame: CGRect, cgBounds: CGRect)? in
            guard
                let number = screen.deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? NSNumber
            else {
                return nil
            }

            let displayID = CGDirectDisplayID(number.uint32Value)
            return (
                frame: screen.frame,
                visibleFrame: screen.visibleFrame,
                cgBounds: CGDisplayBounds(displayID)
            )
        }

        let matchedScreen = screens
            .filter { $0.cgBounds.intersects(cgFrame) }
            .max { lhs, rhs in
                lhs.cgBounds.intersection(cgFrame).width * lhs.cgBounds.intersection(cgFrame).height
                    < rhs.cgBounds.intersection(cgFrame).width * rhs.cgBounds.intersection(cgFrame).height
            }

        guard let matchedScreen else {
            let mainVisibleFrame =
                NSScreen.main?.visibleFrame ?? CGRect(origin: .zero, size: cgFrame.size)
            return (frame: cgFrame, visibleFrame: mainVisibleFrame)
        }

        let localX = cgFrame.minX - matchedScreen.cgBounds.minX
        let localY = cgFrame.minY - matchedScreen.cgBounds.minY
        let frame = CGRect(
            x: matchedScreen.frame.minX + localX,
            y: matchedScreen.frame.maxY - localY - cgFrame.height,
            width: cgFrame.width,
            height: cgFrame.height
        )

        return (frame: frame, visibleFrame: matchedScreen.visibleFrame)
    }
}

@MainActor
private final class PermissionAssistant {
    static let shared = PermissionAssistant()

    private var overlayController: PermissionOverlayWindowController?
    private var trackingTimer: Timer?
    private var activationObserver: NSObjectProtocol?

    func present(panel: PermissionPanel) {
        dismiss()

        let controller = PermissionOverlayWindowController(
            hostApp: .current(),
            panel: panel
        ) { [weak self] in
            self?.dismiss()
        }
        overlayController = controller
        NSWorkspace.shared.open(panel.settingsURL)
        startTracking()
    }

    func dismiss() {
        trackingTimer?.invalidate()
        trackingTimer = nil

        if let activationObserver {
            NSWorkspace.shared.notificationCenter.removeObserver(activationObserver)
            self.activationObserver = nil
        }

        overlayController?.close()
        overlayController = nil
    }

    private func startTracking() {
        trackingTimer?.invalidate()
        trackingTimer = Timer.scheduledTimer(withTimeInterval: 0.15, repeats: true) { [weak self] _ in
            Task { @MainActor in
                self?.refreshPosition()
            }
        }

        if let activationObserver {
            NSWorkspace.shared.notificationCenter.removeObserver(activationObserver)
        }

        activationObserver = NSWorkspace.shared.notificationCenter.addObserver(
            forName: NSWorkspace.didActivateApplicationNotification,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            Task { @MainActor in
                self?.refreshPosition()
            }
        }

        refreshPosition()
    }

    private func refreshPosition() {
        guard let snapshot = SettingsWindowLocator.frontmostWindow() else {
            overlayController?.hide()
            return
        }

        overlayController?.updatePosition(
            with: snapshot.frame,
            visibleFrame: snapshot.visibleFrame
        )
    }
}

private final class PermissionOverlayWindowController: NSWindowController {
    private let windowSize: NSSize

    init(hostApp: HostApp, panel: PermissionPanel, onClose: @escaping () -> Void) {
        windowSize = panel.showsDragSource
            ? NSSize(width: 520, height: 180)
            : NSSize(width: 520, height: 154)

        let window = PassiveOverlayPanel(
            contentRect: NSRect(origin: .zero, size: windowSize),
            styleMask: [.borderless, .nonactivatingPanel],
            backing: .buffered,
            defer: false
        )

        super.init(window: window)
        configureWindow(window)
        window.contentView = PermissionOverlayContentView(
            frame: NSRect(origin: .zero, size: windowSize),
            hostApp: hostApp,
            panel: panel,
            onClose: onClose
        )
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func close() {
        window?.orderOut(nil)
        super.close()
    }

    func updatePosition(with settingsFrame: CGRect, visibleFrame: CGRect) {
        guard let window else { return }
        let origin = anchoredOrigin(for: settingsFrame, visibleFrame: visibleFrame)
        window.setFrameOrigin(origin)
        window.orderFrontRegardless()
        window.alphaValue = 1
    }

    func hide() {
        window?.orderOut(nil)
    }

    private func configureWindow(_ window: NSWindow) {
        window.isOpaque = false
        window.backgroundColor = .clear
        window.level = .statusBar
        window.hasShadow = true
        window.hidesOnDeactivate = false
        window.collectionBehavior = [.canJoinAllSpaces, .stationary, .ignoresCycle, .fullScreenAuxiliary]
        window.animationBehavior = .none
    }

    private func anchoredOrigin(for settingsFrame: CGRect, visibleFrame: CGRect) -> NSPoint {
        let sidebarWidth: CGFloat = 170
        let contentMinX = settingsFrame.minX + sidebarWidth
        let contentWidth = max(settingsFrame.width - sidebarWidth, windowSize.width)
        let preferredX = contentMinX + ((contentWidth - windowSize.width) / 2) - 8
        let preferredY = settingsFrame.minY + 18
        let minX = visibleFrame.minX + 8
        let maxX = visibleFrame.maxX - windowSize.width - 8
        let minY = visibleFrame.minY + 8
        let maxY = visibleFrame.maxY - windowSize.height - 8

        return NSPoint(
            x: min(max(preferredX, minX), maxX),
            y: min(max(preferredY, minY), maxY)
        )
    }
}

private final class PassiveOverlayPanel: NSPanel {
    override var canBecomeKey: Bool { false }
    override var canBecomeMain: Bool { false }
}

private final class PermissionOverlayContentView: NSView {
    private let onClose: () -> Void

    init(frame: NSRect, hostApp: HostApp, panel: PermissionPanel, onClose: @escaping () -> Void) {
        self.onClose = onClose
        super.init(frame: frame)
        translatesAutoresizingMaskIntoConstraints = false
        setup(hostApp: hostApp, panel: panel)
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setup(hostApp: HostApp, panel: PermissionPanel) {
        let materialView = NSVisualEffectView()
        materialView.translatesAutoresizingMaskIntoConstraints = false
        materialView.material = .popover
        materialView.blendingMode = .behindWindow
        materialView.state = .active
        materialView.wantsLayer = true
        materialView.layer?.cornerRadius = 20
        materialView.layer?.masksToBounds = true
        materialView.layer?.borderWidth = 0.5
        materialView.layer?.borderColor = NSColor.separatorColor.withAlphaComponent(0.18).cgColor
        addSubview(materialView)

        let tintView = NSView()
        tintView.translatesAutoresizingMaskIntoConstraints = false
        tintView.wantsLayer = true
        tintView.layer?.backgroundColor = NSColor.windowBackgroundColor.withAlphaComponent(0.82).cgColor
        materialView.addSubview(tintView)

        let closeButton = NSButton()
        closeButton.translatesAutoresizingMaskIntoConstraints = false
        closeButton.isBordered = false
        closeButton.image = NSImage(systemSymbolName: "xmark", accessibilityDescription: "Close")
        closeButton.contentTintColor = NSColor.secondaryLabelColor
        closeButton.target = self
        closeButton.action = #selector(closePressed)
        if let cell = closeButton.cell as? NSButtonCell {
            cell.imagePosition = .imageOnly
        }
        materialView.addSubview(closeButton)

        let iconChrome = NSView()
        iconChrome.translatesAutoresizingMaskIntoConstraints = false
        iconChrome.wantsLayer = true
        iconChrome.layer?.cornerRadius = 14
        iconChrome.layer?.backgroundColor = NSColor.controlBackgroundColor.withAlphaComponent(0.92).cgColor
        materialView.addSubview(iconChrome)

        let iconView = NSImageView(image: hostApp.icon)
        iconView.translatesAutoresizingMaskIntoConstraints = false
        iconView.imageScaling = .scaleProportionallyUpOrDown
        iconChrome.addSubview(iconView)

        let titleLabel = NSTextField(labelWithString: panel.title)
        titleLabel.translatesAutoresizingMaskIntoConstraints = false
        titleLabel.font = .systemFont(ofSize: 16, weight: .semibold)
        titleLabel.textColor = NSColor.labelColor.withAlphaComponent(0.92)
        materialView.addSubview(titleLabel)

        let messageLabel = NSTextField(wrappingLabelWithString: panel.message)
        messageLabel.translatesAutoresizingMaskIntoConstraints = false
        messageLabel.font = .systemFont(ofSize: 13)
        messageLabel.textColor = NSColor.secondaryLabelColor
        messageLabel.maximumNumberOfLines = 3
        messageLabel.lineBreakMode = .byWordWrapping
        materialView.addSubview(messageLabel)

        let instructionRow: NSView = panel.showsDragSource
            ? AppDragSourceView(hostApp: hostApp)
            : AppListRowView(hostApp: hostApp)
        materialView.addSubview(instructionRow)

        NSLayoutConstraint.activate([
            materialView.leadingAnchor.constraint(equalTo: leadingAnchor),
            materialView.trailingAnchor.constraint(equalTo: trailingAnchor),
            materialView.topAnchor.constraint(equalTo: topAnchor),
            materialView.bottomAnchor.constraint(equalTo: bottomAnchor),

            tintView.leadingAnchor.constraint(equalTo: materialView.leadingAnchor),
            tintView.trailingAnchor.constraint(equalTo: materialView.trailingAnchor),
            tintView.topAnchor.constraint(equalTo: materialView.topAnchor),
            tintView.bottomAnchor.constraint(equalTo: materialView.bottomAnchor),

            closeButton.topAnchor.constraint(equalTo: materialView.topAnchor, constant: 14),
            closeButton.trailingAnchor.constraint(equalTo: materialView.trailingAnchor, constant: -14),
            closeButton.widthAnchor.constraint(equalToConstant: 16),
            closeButton.heightAnchor.constraint(equalToConstant: 16),

            iconChrome.leadingAnchor.constraint(equalTo: materialView.leadingAnchor, constant: 20),
            iconChrome.topAnchor.constraint(equalTo: materialView.topAnchor, constant: 20),
            iconChrome.widthAnchor.constraint(equalToConstant: 48),
            iconChrome.heightAnchor.constraint(equalToConstant: 48),

            iconView.centerXAnchor.constraint(equalTo: iconChrome.centerXAnchor),
            iconView.centerYAnchor.constraint(equalTo: iconChrome.centerYAnchor),
            iconView.widthAnchor.constraint(equalToConstant: 36),
            iconView.heightAnchor.constraint(equalToConstant: 36),

            titleLabel.leadingAnchor.constraint(equalTo: iconChrome.trailingAnchor, constant: 14),
            titleLabel.trailingAnchor.constraint(equalTo: closeButton.leadingAnchor, constant: -12),
            titleLabel.topAnchor.constraint(equalTo: materialView.topAnchor, constant: 22),

            messageLabel.leadingAnchor.constraint(equalTo: titleLabel.leadingAnchor),
            messageLabel.trailingAnchor.constraint(equalTo: materialView.trailingAnchor, constant: -20),
            messageLabel.topAnchor.constraint(equalTo: titleLabel.bottomAnchor, constant: 6),

            instructionRow.leadingAnchor.constraint(equalTo: materialView.leadingAnchor, constant: 20),
            instructionRow.trailingAnchor.constraint(equalTo: materialView.trailingAnchor, constant: -20),
            instructionRow.topAnchor.constraint(equalTo: messageLabel.bottomAnchor, constant: 16),
            instructionRow.bottomAnchor.constraint(equalTo: materialView.bottomAnchor, constant: -20),
        ])
    }

    @objc
    private func closePressed() {
        onClose()
    }
}

private final class AppListRowView: NSView {
    init(hostApp: HostApp) {
        super.init(frame: .zero)
        translatesAutoresizingMaskIntoConstraints = false
        setup(hostApp: hostApp)
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setup(hostApp: HostApp) {
        wantsLayer = true
        layer?.cornerRadius = 12
        layer?.backgroundColor = NSColor.controlBackgroundColor.withAlphaComponent(0.85).cgColor
        layer?.borderWidth = 1
        layer?.borderColor = NSColor.separatorColor.withAlphaComponent(0.14).cgColor

        let iconChrome = NSView()
        iconChrome.translatesAutoresizingMaskIntoConstraints = false
        iconChrome.wantsLayer = true
        iconChrome.layer?.cornerRadius = 9
        iconChrome.layer?.backgroundColor = NSColor.white.withAlphaComponent(0.9).cgColor
        addSubview(iconChrome)

        let iconView = NSImageView(image: hostApp.icon)
        iconView.translatesAutoresizingMaskIntoConstraints = false
        iconView.imageScaling = .scaleProportionallyUpOrDown
        iconChrome.addSubview(iconView)

        let label = NSTextField(labelWithString: hostApp.displayName)
        label.translatesAutoresizingMaskIntoConstraints = false
        label.font = .systemFont(ofSize: 14, weight: .semibold)
        label.textColor = NSColor.labelColor.withAlphaComponent(0.85)
        addSubview(label)

        NSLayoutConstraint.activate([
            heightAnchor.constraint(equalToConstant: 48),

            iconChrome.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 12),
            iconChrome.centerYAnchor.constraint(equalTo: centerYAnchor),
            iconChrome.widthAnchor.constraint(equalToConstant: 30),
            iconChrome.heightAnchor.constraint(equalToConstant: 30),

            iconView.centerXAnchor.constraint(equalTo: iconChrome.centerXAnchor),
            iconView.centerYAnchor.constraint(equalTo: iconChrome.centerYAnchor),
            iconView.widthAnchor.constraint(equalToConstant: 24),
            iconView.heightAnchor.constraint(equalToConstant: 24),

            label.leadingAnchor.constraint(equalTo: iconChrome.trailingAnchor, constant: 12),
            label.trailingAnchor.constraint(lessThanOrEqualTo: trailingAnchor, constant: -12),
            label.centerYAnchor.constraint(equalTo: centerYAnchor),
        ])
    }
}

private final class AppDragSourceView: NSView, NSPasteboardItemDataProvider, NSDraggingSource {
    private let hostApp: HostApp
    private let rowView = NSView()
    private let iconChrome = NSView()
    private let label = NSTextField(labelWithString: "")

    init(hostApp: HostApp) {
        self.hostApp = hostApp
        super.init(frame: .zero)
        translatesAutoresizingMaskIntoConstraints = false
        setup()
        updateAppearance()
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func acceptsFirstMouse(for event: NSEvent?) -> Bool {
        true
    }

    override func mouseDown(with event: NSEvent) {
        let pasteboardItem = NSPasteboardItem()
        pasteboardItem.setDataProvider(self, forTypes: [.fileURL])

        let draggingItem = NSDraggingItem(pasteboardWriter: pasteboardItem)
        draggingItem.setDraggingFrame(draggingFrame(), contents: draggingImage())

        let session = beginDraggingSession(with: [draggingItem], event: event, source: self)
        session.animatesToStartingPositionsOnCancelOrFail = true
    }

    override func viewDidChangeEffectiveAppearance() {
        super.viewDidChangeEffectiveAppearance()
        updateAppearance()
    }

    func pasteboard(_ pasteboard: NSPasteboard?, item: NSPasteboardItem, provideDataForType type: NSPasteboard.PasteboardType) {
        guard type == .fileURL else { return }
        item.setData(hostApp.bundleURL.dataRepresentation, forType: .fileURL)
    }

    func draggingSession(_ session: NSDraggingSession, willBeginAt screenPoint: NSPoint) {
        rowView.isHidden = true
    }

    func draggingSession(_ session: NSDraggingSession, sourceOperationMaskFor context: NSDraggingContext) -> NSDragOperation {
        .copy
    }

    func draggingSession(_ session: NSDraggingSession, endedAt screenPoint: NSPoint, operation: NSDragOperation) {
        rowView.isHidden = false
    }

    private func setup() {
        wantsLayer = true

        rowView.wantsLayer = true
        rowView.layer?.cornerRadius = 10
        rowView.layer?.borderWidth = 1
        rowView.translatesAutoresizingMaskIntoConstraints = false
        addSubview(rowView)

        iconChrome.wantsLayer = true
        iconChrome.layer?.backgroundColor = NSColor.white.withAlphaComponent(0.9).cgColor
        iconChrome.layer?.cornerRadius = 8
        iconChrome.translatesAutoresizingMaskIntoConstraints = false
        rowView.addSubview(iconChrome)

        let iconView = NSImageView(image: hostApp.icon)
        iconView.translatesAutoresizingMaskIntoConstraints = false
        iconView.imageScaling = .scaleProportionallyUpOrDown
        iconChrome.addSubview(iconView)

        label.stringValue = hostApp.displayName
        label.font = .systemFont(ofSize: 14, weight: .semibold)
        label.textColor = NSColor.labelColor.withAlphaComponent(0.82)
        label.translatesAutoresizingMaskIntoConstraints = false
        rowView.addSubview(label)

        NSLayoutConstraint.activate([
            rowView.leadingAnchor.constraint(equalTo: leadingAnchor),
            rowView.trailingAnchor.constraint(equalTo: trailingAnchor),
            rowView.topAnchor.constraint(equalTo: topAnchor),
            rowView.bottomAnchor.constraint(equalTo: bottomAnchor),
            rowView.heightAnchor.constraint(equalToConstant: 48),

            iconChrome.leadingAnchor.constraint(equalTo: rowView.leadingAnchor, constant: 12),
            iconChrome.centerYAnchor.constraint(equalTo: rowView.centerYAnchor),
            iconChrome.widthAnchor.constraint(equalToConstant: 30),
            iconChrome.heightAnchor.constraint(equalToConstant: 30),

            iconView.centerXAnchor.constraint(equalTo: iconChrome.centerXAnchor),
            iconView.centerYAnchor.constraint(equalTo: iconChrome.centerYAnchor),
            iconView.widthAnchor.constraint(equalToConstant: 24),
            iconView.heightAnchor.constraint(equalToConstant: 24),

            label.leadingAnchor.constraint(equalTo: iconChrome.trailingAnchor, constant: 12),
            label.trailingAnchor.constraint(lessThanOrEqualTo: rowView.trailingAnchor, constant: -12),
            label.centerYAnchor.constraint(equalTo: rowView.centerYAnchor),
        ])
    }

    private func updateAppearance() {
        let isDark = effectiveAppearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
        if isDark {
            rowView.layer?.backgroundColor = NSColor.white.withAlphaComponent(0.06).cgColor
            rowView.layer?.borderColor = NSColor.white.withAlphaComponent(0.08).cgColor
        } else {
            rowView.layer?.backgroundColor = NSColor.white.withAlphaComponent(0.65).cgColor
            rowView.layer?.borderColor = NSColor(
                red: 0.87451,
                green: 0.866667,
                blue: 0.862745,
                alpha: 1
            ).cgColor
        }
    }

    private func draggingFrame() -> NSRect {
        convert(rowView.bounds, from: rowView)
    }

    private func draggingImage() -> NSImage {
        let image = NSImage(size: rowView.bounds.size)
        image.lockFocus()
        rowView.displayIgnoringOpacity(rowView.bounds, in: NSGraphicsContext.current!)
        image.unlockFocus()
        return image
    }
}

@_cdecl("permission_assistant_present")
func permissionAssistantPresent(_ panelRawValue: Int32) {
    guard let panel = PermissionPanel(rawValue: panelRawValue) else {
        return
    }

    Task { @MainActor in
        PermissionAssistant.shared.present(panel: panel)
    }
}

@_cdecl("permission_assistant_dismiss")
func permissionAssistantDismiss() {
    Task { @MainActor in
        PermissionAssistant.shared.dismiss()
    }
}
