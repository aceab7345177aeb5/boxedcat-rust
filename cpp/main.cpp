#include <QApplication>
#include <QMainWindow>
#include <QWidget>
#include <QHBoxLayout>
#include <QVBoxLayout>
#include <QLabel>
#include <QListWidget>
#include <QStackedWidget>
#include <QTableWidget>
#include <QPushButton>
#include <QHeaderView>
#include <QSplitter>
#include <QCheckBox>
#include <QString>
#include <QGroupBox>
#include <QSplashScreen>
#ifdef _WIN32
#include <windows.h>
#include <shellapi.h>
#endif
#include <QFont>
#include <QStatusBar>
#include <QMessageBox>
#include <QTimer>
#include <QFile>
#include <QTextStream>
#include <QDir>
#include <QStandardPaths>
extern "C" {
    struct BcServiceInfo {
        const char* name;
        const char* display_name;
        unsigned short port;
        const char* status;
        const char* version;
        const char* config_path;
    };
    struct BcPortInfo {
        const char* service;
        unsigned short primary;
        const char* fallbacks;
    };
    struct BcServiceList {
        const BcServiceInfo* items;
        unsigned long count;
    };
    void boxedcat_init();
    BcServiceList boxedcat_get_services();
    unsigned long boxedcat_get_port_count();
    const BcPortInfo* boxedcat_get_port(unsigned long index);
    const char* boxedcat_get_version();
    bool boxedcat_service_start(const char* name);
    bool boxedcat_service_stop(const char* name);
    bool boxedcat_service_is_running(const char* name);
    const char* boxedcat_refresh_service(const char* name);
    struct BcPortStatus {
        unsigned short port;
        bool in_use;
        const char* process_name;
        unsigned int pid;
    };
    BcPortStatus boxedcat_scan_port(unsigned short port);
    int boxedcat_scan_port_count();
    BcPortStatus boxedcat_scan_port_get(int index);
    void boxedcat_free_port_status(BcPortStatus status);
    struct BcProject {
        const char* name;
        const char* path;
        const char* server_name;
        bool has_entry_point;
        const char* project_type;
    };
    struct BcProjectList {
        const BcProject* items;
        unsigned long count;
    };
    BcProjectList boxedcat_scan_projects();
    void boxedcat_free_projects(BcProjectList list);
    bool boxedcat_create_vhost(const char* name, unsigned short port);
    bool boxedcat_delete_vhost(const char* name);
    bool boxedcat_dns_start(unsigned short port, const char* upstream);
    void boxedcat_dns_stop();
    bool boxedcat_dns_is_running();
    const char* boxedcat_get_local_ip();
    const char* boxedcat_get_adapter_name();
    const char* boxedcat_get_hostname();
    unsigned long long boxedcat_get_total_ram_mb();
    const char* boxedcat_get_motherboard();
    bool boxedcat_download_file(const char* url, const char* dest_path);
    const char* boxedcat_save_dialog(const char* default_name);
    const char* boxedcat_browse_folder(const char* title);
    void boxedcat_free_string(char* s);
    void boxedcat_shutdown();
    bool boxedcat_generate_cert();
    bool boxedcat_tls_is_configured();
    const char* boxedcat_get_certs_dir();
    bool boxedcat_delete_cert();
}
static QApplication* g_app = nullptr;
static QString themeConfigPath() {
    QString dataDir = QCoreApplication::applicationDirPath();
    return dataDir + "/theme.conf";
}
static bool loadDarkMode() {
    QFile f(themeConfigPath());
    if (!f.open(QIODevice::ReadOnly | QIODevice::Text)) return false;
    QString line = f.readAll().trimmed().toLower();
    f.close();
    return line == "dark" || line == "1" || line == "true";
}
static void saveDarkMode(bool dark) {
    QFile f(themeConfigPath());
    if (f.open(QIODevice::WriteOnly | QIODevice::Text)) {
        f.write(dark ? "dark\n" : "light\n");
    }
}
static QString lightCss() {
    return
        "*{font-family:\"Segoe UI\",sans-serif;font-size:13px}"
        "QMainWindow{background-color:#fcfcfd}"
        "QWidget{background-color:#fcfcfd;color:#1c2024}"
        "QListWidget{background-color:#f9f9fb;border:none;border-right:1px solid #d9d9e0;outline:none;padding:4px}"
        "QListWidget::item{padding:8px 12px;border-radius:6px;color:#1c2024}"
        "QListWidget::item:selected{background-color:#e8e8ec;color:#1c2024}"
        "QListWidget::item:hover{background-color:#f0f0f3}"
        "QPushButton{background-color:#f0f0f3;color:#1c2024;border:1px solid #cdced6;border-radius:6px;padding:6px 14px;min-height:20px}"
        "QPushButton:hover{background-color:#e8e8ec;border-color:#b9bbc6}"
        "QPushButton:pressed{background-color:#e0e1e6}"
        "QPushButton[objectName=\"primary\"]{background-color:#3b82f6;color:white;border:none}"
        "QPushButton[objectName=\"primary\"]:hover{background-color:#60a5fa}"
        "QPushButton[objectName=\"start\"]{background-color:#30a46c;color:white;border:none}"
        "QPushButton[objectName=\"start\"]:hover{background-color:#3cb578}"
        "QPushButton[objectName=\"stop\"]{background-color:#e5484d;color:white;border:none}"
        "QPushButton[objectName=\"stop\"]:hover{background-color:#ec6468}"
        "QPushButton[objectName=\"restart\"]{background-color:#f5a623;color:white;border:none}"
        "QPushButton[objectName=\"restart\"]:hover{background-color:#f7b84e}"
        "QPushButton[objectName=\"danger\"]{background-color:#e5484d;color:white;border:none}"
        "QPushButton[objectName=\"danger\"]:hover{background-color:#ec6468}"
        "QGroupBox{border:1px solid #d9d9e0;border-radius:8px;margin-top:8px;padding:12px}"
        "QGroupBox::title{subcontrol-origin:margin;left:12px;padding:0 4px;color:#60646c;font-weight:600}"
        "QTableView,QTableWidget{background-color:#ffffff;border:1px solid #d9d9e0;border-radius:8px;gridline-color:#e8e8ec;selection-background-color:#e8e8ec;selection-color:#1c2024}"
        "QHeaderView::section{background-color:#f9f9fb;color:#60646c;border:none;border-bottom:1px solid #d9d9e0;padding:8px 12px;font-weight:600}"
        "QLineEdit,QSpinBox,QComboBox{background-color:#ffffff;color:#1c2024;border:1px solid #cdced6;border-radius:6px;padding:6px 10px;min-height:20px}"
        "QLineEdit:focus,QSpinBox:focus,QComboBox:focus{border-color:#3b82f6}"
        "QScrollBar:vertical{background:transparent;width:8px}"
        "QScrollBar::handle:vertical{background-color:#cdced6;border-radius:4px;min-height:24px}"
        "QScrollBar::handle:vertical:hover{background-color:#b9bbc6}"
        "QScrollBar::add-line,QScrollBar::sub-line{height:0px}"
        "QStatusBar{background-color:#f9f9fb;color:#60646c;border-top:1px solid #d9d9e0}"
        "#healthCard{background-color:#ffffff;border:1px solid #d9d9e0;border-radius:8px}";
}
static QString darkCss() {
    return
        "*{font-family:\"Segoe UI\",sans-serif;font-size:13px}"
        "QMainWindow{background-color:#111113}"
        "QWidget{background-color:#111113;color:#edeeef}"
        "QListWidget{background-color:#18191b;border:none;border-right:1px solid #43484e;outline:none;padding:4px}"
        "QListWidget::item{padding:8px 12px;border-radius:6px;color:#edeeef}"
        "QListWidget::item:selected{background-color:#272a2d;color:#edeeef}"
        "QListWidget::item:hover{background-color:#232527}"
        "QPushButton{background-color:#232527;color:#edeeef;border:1px solid #43484e;border-radius:6px;padding:6px 14px;min-height:20px}"
        "QPushButton:hover{background-color:#272a2d;border-color:#5a6169}"
        "QPushButton:pressed{background-color:#2e3033}"
        "QPushButton[objectName=\"primary\"]{background-color:#3b82f6;color:white;border:none}"
        "QPushButton[objectName=\"primary\"]:hover{background-color:#60a5fa}"
        "QPushButton[objectName=\"start\"]{background-color:#30a46c;color:white;border:none}"
        "QPushButton[objectName=\"start\"]:hover{background-color:#3cb578}"
        "QPushButton[objectName=\"stop\"]{background-color:#e5484d;color:white;border:none}"
        "QPushButton[objectName=\"stop\"]:hover{background-color:#ec6468}"
        "QPushButton[objectName=\"restart\"]{background-color:#f5a623;color:white;border:none}"
        "QPushButton[objectName=\"restart\"]:hover{background-color:#f7b84e}"
        "QPushButton[objectName=\"danger\"]{background-color:#e5484d;color:white;border:none}"
        "QPushButton[objectName=\"danger\"]:hover{background-color:#ec6468}"
        "QGroupBox{border:1px solid #43484e;border-radius:8px;margin-top:8px;padding:12px}"
        "QGroupBox::title{subcontrol-origin:margin;left:12px;padding:0 4px;color:#8b8d98;font-weight:600}"
        "QTableView,QTableWidget{background-color:#18191b;border:1px solid #43484e;border-radius:8px;gridline-color:#2e3033;selection-background-color:#272a2d;selection-color:#edeeef}"
        "QHeaderView::section{background-color:#18191b;color:#8b8d98;border:none;border-bottom:1px solid #43484e;padding:8px 12px;font-weight:600}"
        "QLineEdit,QSpinBox,QComboBox{background-color:#18191b;color:#edeeef;border:1px solid #43484e;border-radius:6px;padding:6px 10px;min-height:20px}"
        "QLineEdit:focus,QSpinBox:focus,QComboBox:focus{border-color:#3b82f6}"
        "QScrollBar:vertical{background:transparent;width:8px}"
        "QScrollBar::handle:vertical{background-color:#43484e;border-radius:4px;min-height:24px}"
        "QScrollBar::handle:vertical:hover{background-color:#5a6169}"
        "QScrollBar::add-line,QScrollBar::sub-line{height:0px}"
        "QStatusBar{background-color:#18191b;color:#8b8d98;border-top:1px solid #43484e}"
        "#healthCard{background-color:#18191b;border:1px solid #43484e;border-radius:8px}";
}
static void applyTheme(bool dark) {
    if (!g_app) return;
    g_app->setStyleSheet(dark ? darkCss() : lightCss());
}
static void addRow(QTableWidget* t, const char* a, const char* b, const char* c, const char* d = nullptr) {
    if (!t) return;
    int r = t->rowCount(); t->insertRow(r);
    t->setItem(r, 0, new QTableWidgetItem(a));
    t->setItem(r, 1, new QTableWidgetItem(b));
    t->setItem(r, 2, new QTableWidgetItem(c));
    if (d) t->setItem(r, 3, new QTableWidgetItem(d));
}
struct ServiceContext {
    const char* name;
    QTableWidget* table;
    int row;
};
static void refreshServiceStatus(ServiceContext ctx) {
    const char* status = boxedcat_refresh_service(ctx.name);
    ctx.table->setItem(ctx.row, 1, new QTableWidgetItem(status));
}
int main(int argc, char** argv) {
    QApplication app(argc, argv);
    g_app = &app;
#ifdef _WIN32
    {
        typedef HRESULT (WINAPI *SetAppUserModelID)(PCWSTR);
        HMODULE shell32 = LoadLibraryW(L"shell32.dll");
        if (shell32) {
            auto fn = (SetAppUserModelID)GetProcAddress(shell32, "SetCurrentProcessExplicitAppUserModelID");
            if (fn) fn(L"BoxedCat.DevManager");
            FreeLibrary(shell32);
        }
    }
#endif
    QString appDir = QCoreApplication::applicationDirPath();
    QIcon appIcon(":/icons/boxedcat.ico");
    if (appIcon.isNull()) appIcon = QIcon(":/icons/boxedcat_128.png");
    if (appIcon.isNull()) {
        QStringList icoSearch = {
            appDir + "/assets/icons/boxedcat.ico",
            appDir + "/../assets/icons/boxedcat.ico",
        };
        for (const QString& p : icoSearch) {
            if (QFile::exists(p)) { appIcon = QIcon(p); break; }
        }
    }
    if (!appIcon.isNull()) {
        app.setWindowIcon(appIcon);
    }
    bool darkMode = loadDarkMode();
    applyTheme(darkMode);
    QSplashScreen* splash = nullptr;
    QPixmap splashPix(":/icons/splash.png");
    if (!splashPix.isNull()) {
        splash = new QSplashScreen(splashPix);
        splash->show();
        app.processEvents();
    }
    boxedcat_init();
    QMainWindow w;
    w.setWindowTitle("BoxedCat");
    if (!appIcon.isNull()) w.setWindowIcon(appIcon);
    w.setMinimumSize(644, 480);
    auto* central = new QWidget();
    auto* mainLayout = new QHBoxLayout(central);
    mainLayout->setSpacing(0);
    mainLayout->setContentsMargins(0,0,0,0);
    auto* splitter = new QSplitter(Qt::Horizontal);
    auto* sidebar = new QWidget();
    sidebar->setFixedWidth(160);
    sidebar->setObjectName("sidebar");
    auto* sbl = new QVBoxLayout(sidebar);
    sbl->setSpacing(4);
    sbl->setContentsMargins(8,12,8,12);
    auto* nav = new QListWidget();
    nav->setObjectName("nav");
    nav->addItem("Dashboard"); nav->addItem("Services"); nav->addItem("Ports");
    nav->addItem("Projects"); nav->addItem("Versions"); nav->addItem("Settings"); nav->addItem("About");
    nav->setCurrentRow(0);
    nav->setStyleSheet("QListWidget::item { padding: 6px 8px; }");
    sbl->addWidget(nav);
    sbl->addStretch();
    splitter->addWidget(sidebar);
    auto* content = new QStackedWidget();
    auto* dash = new QWidget();
    auto* dl = new QVBoxLayout(dash);
    dl->setSpacing(12);
    dl->setContentsMargins(16,16,16,16);
    auto* dashHeader = new QWidget();
    auto* dashHeaderLayout = new QHBoxLayout(dashHeader);
    dashHeaderLayout->setContentsMargins(0,0,0,0);
    auto* sysInfoLabel = new QLabel("System Health Monitor");
    QFont dhFont = sysInfoLabel->font(); dhFont.setPointSize(16); dhFont.setBold(true);
    sysInfoLabel->setFont(dhFont);
    dashHeaderLayout->addWidget(sysInfoLabel);
    dashHeaderLayout->addStretch();
    const char* localIp = boxedcat_get_local_ip();
    const char* adapterName = boxedcat_get_adapter_name();
    QString ipText = QString("Local: 127.0.0.1 | Network: %1 (%2)")
        .arg(QString::fromUtf8(localIp))
        .arg(QString::fromUtf8(adapterName));
    auto* ipLabel = new QLabel(ipText);
    QFont ipFont = ipLabel->font(); ipFont.setPointSize(10);
    ipLabel->setFont(ipFont);
    ipLabel->setStyleSheet("color: #8b8d98;");
    dashHeaderLayout->addWidget(ipLabel);
    dl->addWidget(dashHeader);
    auto* cardsWidget = new QWidget();
    auto* cardsLayout = new QHBoxLayout(cardsWidget);
    cardsLayout->setSpacing(12);
    cardsLayout->setContentsMargins(0,0,0,0);
    auto* svcHealthCard = new QWidget();
    svcHealthCard->setObjectName("healthCard");
    auto* svcHealthLayout = new QVBoxLayout(svcHealthCard);
    svcHealthLayout->setContentsMargins(12,12,12,12);
    auto* svcHealthLayout2 = new QVBoxLayout();
    svcHealthLayout2->setAlignment(Qt::AlignCenter);
    auto* svcHealthStatus = new QLabel("3/3");
    QFont cardNumberFont = svcHealthStatus->font(); cardNumberFont.setPointSize(22); cardNumberFont.setBold(true);
    svcHealthStatus->setFont(cardNumberFont);
    svcHealthStatus->setStyleSheet("color: #30a46c;");
    svcHealthStatus->setAlignment(Qt::AlignCenter);
    svcHealthLayout2->addWidget(svcHealthStatus);
    auto* svcHealthLabel = new QLabel("Running Services");
    QFont cardLabelFont = svcHealthLabel->font(); cardLabelFont.setPointSize(10);
    svcHealthLabel->setFont(cardLabelFont);
    svcHealthLabel->setStyleSheet("color: #60646c;");
    svcHealthLabel->setAlignment(Qt::AlignCenter);
    svcHealthLayout2->addWidget(svcHealthLabel);
    svcHealthLayout->addLayout(svcHealthLayout2);
    svcHealthLayout->addStretch();
    cardsLayout->addWidget(svcHealthCard);
    auto* portHealthCard = new QWidget();
    portHealthCard->setObjectName("healthCard");
    auto* portHealthLayout = new QVBoxLayout(portHealthCard);
    portHealthLayout->setContentsMargins(12,12,12,12);
    auto* portHealthLayout2 = new QVBoxLayout();
    portHealthLayout2->setAlignment(Qt::AlignCenter);
    auto* portHealthStatus = new QLabel("10/10");
    portHealthStatus->setFont(cardNumberFont);
    portHealthStatus->setStyleSheet("color: #3b82f6;");
    portHealthStatus->setAlignment(Qt::AlignCenter);
    portHealthLayout2->addWidget(portHealthStatus);
    auto* portHealthLabel = new QLabel("Available Ports");
    portHealthLabel->setFont(cardLabelFont);
    portHealthLabel->setStyleSheet("color: #60646c;");
    portHealthLabel->setAlignment(Qt::AlignCenter);
    portHealthLayout2->addWidget(portHealthLabel);
    portHealthLayout->addLayout(portHealthLayout2);
    portHealthLayout->addStretch();
    cardsLayout->addWidget(portHealthCard);
    auto* dnsHealthCard = new QWidget();
    dnsHealthCard->setObjectName("healthCard");
    auto* dnsHealthLayout = new QVBoxLayout(dnsHealthCard);
    dnsHealthLayout->setContentsMargins(12,12,12,12);
    auto* dnsHealthLayout2 = new QVBoxLayout();
    dnsHealthLayout2->setAlignment(Qt::AlignCenter);
    auto* dnsHealthStatus = new QLabel("5399");
    dnsHealthStatus->setFont(cardNumberFont);
    dnsHealthStatus->setStyleSheet("color: #e5484d;");
    dnsHealthStatus->setAlignment(Qt::AlignCenter);
    dnsHealthLayout2->addWidget(dnsHealthStatus);
    auto* dnsHealthLabel = new QLabel("DNS Proxy Port");
    dnsHealthLabel->setFont(cardLabelFont);
    dnsHealthLabel->setStyleSheet("color: #60646c;");
    dnsHealthLabel->setAlignment(Qt::AlignCenter);
    dnsHealthLayout2->addWidget(dnsHealthLabel);
    auto* dnsHealthState = new QLabel("Stopped");
    dnsHealthState->setFont(cardLabelFont);
    dnsHealthState->setStyleSheet("color: #e5484d;");
    dnsHealthState->setAlignment(Qt::AlignCenter);
    dnsHealthLayout2->addWidget(dnsHealthState);
    dnsHealthLayout->addLayout(dnsHealthLayout2);
    dnsHealthLayout->addStretch();
    cardsLayout->addWidget(dnsHealthCard);
    auto refreshDnsHealth = [dnsHealthState]() {
        bool running = boxedcat_dns_is_running();
        dnsHealthState->setText(running ? "Running" : "Stopped");
        dnsHealthState->setStyleSheet(running ? "color: #30a46c;" : "color: #e5484d;");
    };
    refreshDnsHealth();
    dl->addWidget(cardsWidget);
    auto* hwWidget = new QWidget();
    auto* hwLayout = new QHBoxLayout(hwWidget);
    hwLayout->setContentsMargins(0,0,0,0);
    hwLayout->setSpacing(24);
    const char* hostname = boxedcat_get_hostname();
    const char* motherboard = boxedcat_get_motherboard();
    unsigned long long ram_mb = boxedcat_get_total_ram_mb();
    const char* nic_name = boxedcat_get_adapter_name();
    const char* nic_ip = boxedcat_get_local_ip();
    auto* hwTitle = new QLabel("Workstation");
    QFont hwTitleFont = hwTitle->font(); hwTitleFont.setPointSize(12); hwTitleFont.setBold(true);
    hwTitle->setFont(hwTitleFont);
    hwLayout->addWidget(hwTitle);
    auto* hwHostLabel = new QLabel(QString("Name: %1").arg(QString::fromUtf8(hostname)));
    hwHostLabel->setStyleSheet("color: #60646c; font-size: 10px;");
    hwLayout->addWidget(hwHostLabel);
    auto* hwMbLabel = new QLabel(QString("Board: %1").arg(QString::fromUtf8(motherboard)));
    hwMbLabel->setStyleSheet("color: #60646c; font-size: 10px;");
    hwLayout->addWidget(hwMbLabel);
    double ram_gb = ram_mb / 1024.0;
    auto* hwRamLabel = new QLabel(QString("RAM: %1 MB (%2 GB)").arg(ram_mb).arg(ram_gb, 0, 'f', 1));
    hwRamLabel->setStyleSheet("color: #60646c; font-size: 10px;");
    hwLayout->addWidget(hwRamLabel);
    auto* hwNicLabel = new QLabel(QString("NIC: %1").arg(QString::fromUtf8(nic_name)));
    hwNicLabel->setStyleSheet("color: #60646c; font-size: 10px;");
    hwLayout->addWidget(hwNicLabel);
    hwLayout->addStretch();
    dl->addWidget(hwWidget);
    auto* svcStatusTitle = new QLabel("Service Status");
    QFont svcTitleFont = svcStatusTitle->font(); svcTitleFont.setPointSize(12); svcTitleFont.setBold(true);
    svcStatusTitle->setFont(svcTitleFont);
    dl->addWidget(svcStatusTitle);
    auto* dashTable = new QTableWidget();
    dashTable->setColumnCount(5);
    QStringList h1; h1 << "Service" << "Status" << "Port" << "PID" << "Health";
    dashTable->setHorizontalHeaderLabels(h1);
    dashTable->horizontalHeader()->setStretchLastSection(true);
    dashTable->setEditTriggers(QAbstractItemView::NoEditTriggers);
    dashTable->verticalHeader()->setVisible(false);
    dashTable->setSelectionBehavior(QAbstractItemView::SelectRows);
    dashTable->setColumnWidth(0, 140);
    dashTable->setColumnWidth(1, 80);
    dashTable->setColumnWidth(2, 60);
    dashTable->setColumnWidth(3, 60);
    dashTable->setColumnWidth(4, 80);
    dashTable->setRowHeight(0, 36);
    dl->addWidget(dashTable);
    dl->addStretch();
    content->addWidget(dash);
    auto* svcPage = new QWidget();
    auto* sl = new QVBoxLayout(svcPage);
    sl->setSpacing(12);
    sl->setContentsMargins(16,16,16,16);
    auto* svcHeader = new QWidget();
    auto* svcHeaderLayout = new QHBoxLayout(svcHeader);
    svcHeaderLayout->setContentsMargins(0,0,0,0);
    auto* st = new QLabel("Services");
    QFont sf = st->font(); sf.setPointSize(14); sf.setBold(true); st->setFont(sf);
    svcHeaderLayout->addWidget(st);
    svcHeaderLayout->addStretch();
    auto* refreshAllBtn = new QPushButton("\xe2\x8f\xb3 Refresh All");
    refreshAllBtn->setObjectName("primary");
    svcHeaderLayout->addWidget(refreshAllBtn);
    sl->addWidget(svcHeader);
    auto* svcTable = new QTableWidget();
    svcTable->setColumnCount(5);
    QStringList h2; h2 << "Service" << "Status" << "Port" << "Version" << "Controls";
    svcTable->setHorizontalHeaderLabels(h2);
    svcTable->horizontalHeader()->setStretchLastSection(true);
    svcTable->setEditTriggers(QAbstractItemView::NoEditTriggers);
    svcTable->verticalHeader()->setVisible(false);
    svcTable->setColumnWidth(0, 140);
    svcTable->setColumnWidth(1, 80);
    svcTable->setColumnWidth(2, 60);
    svcTable->setColumnWidth(3, 80);
    sl->addWidget(svcTable);
    auto* tlsGroup = new QGroupBox("HTTPS/TLS Certificate");
    auto* tlsLayout = new QVBoxLayout(tlsGroup);
    auto* tlsStatusRow = new QWidget();
    auto* tlsStatusLayout = new QHBoxLayout(tlsStatusRow);
    tlsStatusLayout->setContentsMargins(0,0,0,0);
    auto* tlsStatusLabel = new QLabel("Status:");
    auto* tlsStatusValue = new QLabel("Checking...");
    tlsStatusValue->setObjectName("themeDesc");
    tlsStatusLayout->addWidget(tlsStatusLabel);
    tlsStatusLayout->addWidget(tlsStatusValue);
    tlsStatusLayout->addStretch();
    tlsLayout->addWidget(tlsStatusRow);
    auto* tlsBtnRow = new QWidget();
    auto* tlsBtnLayout = new QHBoxLayout(tlsBtnRow);
    tlsBtnLayout->setContentsMargins(0,0,0,0);
    auto* genCertBtn = new QPushButton("\xe2\x9c\x94 Generate Certificate");
    genCertBtn->setObjectName("primary");
    auto* deleteCertBtn = new QPushButton("\xe2\x9c\x95 Delete Certificate");
    deleteCertBtn->setObjectName("danger");
    tlsBtnLayout->addWidget(genCertBtn);
    tlsBtnLayout->addWidget(deleteCertBtn);
    tlsBtnLayout->addStretch();
    tlsLayout->addWidget(tlsBtnRow);
    auto* tlsDescLabel = new QLabel("Self-signed certificate for localhost development. Apache must be configured with SSL module to use HTTPS.");
    tlsDescLabel->setObjectName("themeDesc");
    tlsDescLabel->setWordWrap(true);
    tlsLayout->addWidget(tlsDescLabel);
    auto refreshTlsStatus = [tlsStatusValue]() {
        bool configured = boxedcat_tls_is_configured();
        tlsStatusValue->setText(configured ? "Configured (server.crt + server.key)" : "No Certificate");
        tlsStatusValue->setStyleSheet(configured ? "color: #30a46c;" : "color: #e5484d;");
    };
    refreshTlsStatus();
    QObject::connect(genCertBtn, &QPushButton::clicked, [refreshTlsStatus]() {
        boxedcat_generate_cert();
        refreshTlsStatus();
    });
    QObject::connect(deleteCertBtn, &QPushButton::clicked, [refreshTlsStatus]() {
        boxedcat_delete_cert();
        refreshTlsStatus();
    });
    sl->addWidget(tlsGroup);
    content->addWidget(svcPage);
    auto* portPage = new QWidget();
    auto* pl = new QVBoxLayout(portPage);
    pl->setSpacing(12);
    pl->setContentsMargins(16,16,16,16);
    auto* pt = new QLabel("Port Manager");
    QFont pf = pt->font(); pf.setPointSize(14); pf.setBold(true); pt->setFont(pf);
    pl->addWidget(pt);
    auto* portTable = new QTableWidget();
    portTable->setColumnCount(5);
    QStringList h3; h3 << "Service" << "Primary" << "Status" << "Process" << "PID";
    portTable->setHorizontalHeaderLabels(h3);
    portTable->horizontalHeader()->setStretchLastSection(true);
    portTable->setEditTriggers(QAbstractItemView::NoEditTriggers);
    portTable->verticalHeader()->setVisible(false);
    portTable->setSelectionBehavior(QAbstractItemView::SelectRows);
    portTable->setAlternatingRowColors(true);
    pl->addWidget(portTable);
    auto* portRefreshBtn = new QPushButton("\xe2\x8f\xb3 Scan Ports");
    portRefreshBtn->setObjectName("primary");
    auto* portBtnLayout = new QHBoxLayout();
    portBtnLayout->addWidget(portRefreshBtn);
    portBtnLayout->addStretch();
    pl->addLayout(portBtnLayout);
    auto populatePorts = [portTable]() {
        portTable->setRowCount(0);
        int count = boxedcat_scan_port_count();
        portTable->setRowCount(count);
        QStringList svcNames = {"HTTP", "HTTPS", "MariaDB", "PostgreSQL", "Redis", "HTTP Alt", "HTTPS Alt", "MariaDB Alt", "MariaDB Alt 2", "MariaDB Alt 3"};
        for (int i = 0; i < count; i++) {
            BcPortStatus ps = boxedcat_scan_port_get(i);
            auto* svcItem = new QTableWidgetItem(svcNames.value(i, QString::number(ps.port)));
            auto* portItem = new QTableWidgetItem(QString::number(ps.port));
            auto* statusItem = new QTableWidgetItem(ps.in_use ? "In Use" : "Available");
            statusItem->setForeground(ps.in_use ? QColor("#e5484d") : QColor("#30a46c"));
            auto* procItem = new QTableWidgetItem(ps.in_use ? QString::fromUtf8(ps.process_name) : "");
            auto* pidItem = new QTableWidgetItem(ps.in_use ? QString::number(ps.pid) : "");
            portTable->setItem(i, 0, svcItem);
            portTable->setItem(i, 1, portItem);
            portTable->setItem(i, 2, statusItem);
            portTable->setItem(i, 3, procItem);
            portTable->setItem(i, 4, pidItem);
            boxedcat_free_port_status(ps);
        }
    };
    populatePorts();
    QObject::connect(portRefreshBtn, &QPushButton::clicked, populatePorts);
    content->addWidget(portPage);
    auto* proj = new QWidget();
    auto* rjl = new QVBoxLayout(proj);
    rjl->setSpacing(12);
    rjl->setContentsMargins(16,16,16,16);
    auto* rjt = new QLabel("Projects");
    QFont rjf = rjt->font(); rjf.setPointSize(14); rjf.setBold(true); rjt->setFont(rjf);
    rjl->addWidget(rjt);
    auto* projTable = new QTableWidget();
    projTable->setColumnCount(4);
    QStringList h4; h4 << "Name" << "Type" << "Server Name" << "Entry Point";
    projTable->setHorizontalHeaderLabels(h4);
    projTable->horizontalHeader()->setStretchLastSection(true);
    projTable->setEditTriggers(QAbstractItemView::NoEditTriggers);
    projTable->verticalHeader()->setVisible(false);
    projTable->setSelectionBehavior(QAbstractItemView::SelectRows);
    projTable->setAlternatingRowColors(true);
    projTable->setColumnWidth(0, 150);
    projTable->setColumnWidth(1, 80);
    projTable->setColumnWidth(2, 140);
    rjl->addWidget(projTable);
    auto* scanProjectsBtn = new QPushButton("\xe2\x8f\xb3 Scan Projects");
    scanProjectsBtn->setObjectName("primary");
    auto populateProjects = [projTable]() {
        projTable->setRowCount(0);
        BcProjectList list = boxedcat_scan_projects();
        projTable->setRowCount(list.count);
        for (unsigned long i = 0; i < list.count; i++) {
            const BcProject& p = list.items[i];
            projTable->setItem(i, 0, new QTableWidgetItem(QString::fromUtf8(p.name)));
            projTable->setItem(i, 1, new QTableWidgetItem(QString::fromUtf8(p.project_type)));
            projTable->setItem(i, 2, new QTableWidgetItem(QString::fromUtf8(p.server_name)));
            auto* entryItem = new QTableWidgetItem(p.has_entry_point ? "Yes" : "No");
            entryItem->setForeground(p.has_entry_point ? QColor("#30a46c") : QColor("#e5484d"));
            projTable->setItem(i, 3, entryItem);
        }
        boxedcat_free_projects(list);
    };
    populateProjects();
    auto* createVhostBtn = new QPushButton("\xe2\x9c\x94 Create Vhost");
    createVhostBtn->setObjectName("primary");
    auto* deleteVhostBtn = new QPushButton("\xe2\x9c\x95 Delete Vhost");
    deleteVhostBtn->setObjectName("danger");
    auto* projBtnLayout = new QHBoxLayout();
    projBtnLayout->addWidget(scanProjectsBtn);
    projBtnLayout->addWidget(createVhostBtn);
    projBtnLayout->addWidget(deleteVhostBtn);
    projBtnLayout->addStretch();
    QObject::connect(scanProjectsBtn, &QPushButton::clicked, populateProjects);
    QObject::connect(createVhostBtn, &QPushButton::clicked, [projTable]() {
        int row = projTable->currentRow();
        if (row < 0) return;
        QString name = projTable->item(row, 0)->text();
        boxedcat_create_vhost(name.toUtf8().constData(), 80);
    });
    QObject::connect(deleteVhostBtn, &QPushButton::clicked, [projTable]() {
        int row = projTable->currentRow();
        if (row < 0) return;
        QString name = projTable->item(row, 0)->text();
        boxedcat_delete_vhost(name.toUtf8().constData());
    });
    rjl->addLayout(projBtnLayout);
    content->addWidget(proj);
    auto* verpg = new QWidget();
    auto* vl = new QVBoxLayout(verpg);
    vl->setSpacing(12);
    vl->setContentsMargins(16,16,16,16);
    auto* vt = new QLabel("Versions");
    QFont vf = vt->font(); vf.setPointSize(14); vf.setBold(true); vt->setFont(vf);
    vl->addWidget(vt);
    vl->addWidget(new QLabel("LTS-only version registry with verified download sources"));
    auto* verTable = new QTableWidget();
    verTable->setColumnCount(4);
    QStringList vh; vh << "Software" << "Version" << "Status" << "Source";
    verTable->setHorizontalHeaderLabels(vh);
    verTable->horizontalHeader()->setStretchLastSection(true);
    verTable->setEditTriggers(QAbstractItemView::NoEditTriggers);
    verTable->verticalHeader()->setVisible(false);
    verTable->setSelectionBehavior(QAbstractItemView::SelectRows);
    verTable->setColumnWidth(0, 100);
    verTable->setColumnWidth(1, 80);
    verTable->setColumnWidth(2, 80);
    vl->addWidget(verTable);
    auto addVerRow = [verTable](const char* name, const char* ver, const char* status, const char* source) {
        int r = verTable->rowCount(); verTable->insertRow(r);
        verTable->setItem(r, 0, new QTableWidgetItem(name));
        verTable->setItem(r, 1, new QTableWidgetItem(ver));
        auto* statusItem = new QTableWidgetItem(status);
        statusItem->setForeground(QColor("#30a46c"));
        verTable->setItem(r, 2, statusItem);
        verTable->setItem(r, 3, new QTableWidgetItem(source));
    };
    addVerRow("Apache", "2.4.66", "Installed", "apachelounge.com");
    addVerRow("MariaDB", "11.4.13", "Installed", "mariadb.org");
    addVerRow("PHP", "8.5.10", "Installed", "windows.php.net");
    addVerRow("PostgreSQL", "Not installed", "Available", "postgresql.org");
    addVerRow("Nginx", "Not installed", "Available", "nginx.org");
    addVerRow("Redis", "Not installed", "Available", "redis.io");
    auto* verBtnLayout = new QHBoxLayout();
    auto* downloadBtn = new QPushButton("\xe2\x8e\x95 Download Selected");
    downloadBtn->setObjectName("primary");
    auto* deployBtn = new QPushButton("\xe2\x9c\xa8 Deploy Selected");
    deployBtn->setObjectName("primary");
    auto* removeBtn = new QPushButton("\xe2\x9c\x95 Remove");
    removeBtn->setObjectName("stop");
    verBtnLayout->addWidget(downloadBtn);
    verBtnLayout->addWidget(deployBtn);
    verBtnLayout->addWidget(removeBtn);
    verBtnLayout->addStretch();
    vl->addLayout(verBtnLayout);
    auto* verDesc = new QLabel("Only LTS and latest stable versions are supported. All download sources are verified for supply chain safety.");
    verDesc->setObjectName("themeDesc");
    vl->addWidget(verDesc);
    vl->addStretch();
    content->addWidget(verpg);
    auto* setpg = new QWidget();
    auto* setl = new QVBoxLayout(setpg);
    setl->setSpacing(12);
    setl->setContentsMargins(16,16,16,16);
    auto* sett = new QLabel("Settings");
    QFont setf = sett->font(); setf.setPointSize(14); setf.setBold(true); sett->setFont(setf);
    setl->addWidget(sett);
    auto* themeGroup = new QWidget();
    auto* themeLayout = new QHBoxLayout(themeGroup);
    themeLayout->setContentsMargins(0,8,0,8);
    auto* themeLabel = new QLabel("Theme");
    QFont tlf = themeLabel->font(); tlf.setPointSize(12); tlf.setBold(true); themeLabel->setFont(tlf);
    themeLayout->addWidget(themeLabel);
    themeLayout->addStretch();
    auto* lightBtn = new QPushButton("Light");
    lightBtn->setObjectName("primary");
    lightBtn->setFixedWidth(80);
    auto* darkBtn = new QPushButton("Dark");
    darkBtn->setObjectName("primary");
    darkBtn->setFixedWidth(80);
    themeLayout->addWidget(lightBtn);
    themeLayout->addWidget(darkBtn);
    auto* themeHint = new QLabel(darkMode ? "Current: Dark" : "Current: Light");
    themeHint->setObjectName("themeHint");
    themeLayout->addWidget(themeHint);
    setl->addWidget(themeGroup);
    auto* themeDesc = new QLabel("Switch between light and dark themes. Preference is saved automatically.");
    themeDesc->setObjectName("themeDesc");
    setl->addWidget(themeDesc);
    auto* xamppGroup = new QWidget();
    auto* xamppLayout = new QHBoxLayout(xamppGroup);
    xamppLayout->setContentsMargins(0,8,0,8);
    auto* xamppLabel = new QLabel("Server Directory");
    QFont xlf = xamppLabel->font(); xlf.setPointSize(12); xlf.setBold(true); xamppLabel->setFont(xlf);
    xamppLayout->addWidget(xamppLabel);
    xamppLayout->addStretch();
    QString serverDir = QCoreApplication::applicationDirPath();
    auto* xamppPath = new QLabel(serverDir);
    xamppPath->setStyleSheet("color: #60646c;");
    xamppLayout->addWidget(xamppPath);
    setl->addWidget(xamppGroup);
    auto* xamppDesc = new QLabel("Base directory for Apache, MariaDB, and PHP. Server files are expected alongside BoxedCat.");
    xamppDesc->setObjectName("themeDesc");
    setl->addWidget(xamppDesc);
    auto* dnsGroup = new QWidget();
    auto* dnsLayout = new QHBoxLayout(dnsGroup);
    dnsLayout->setContentsMargins(0,8,0,8);
    auto* dnsLabel = new QLabel("DNS Proxy");
    QFont dlf = dnsLabel->font(); dlf.setPointSize(12); dlf.setBold(true); dnsLabel->setFont(dlf);
    dnsLayout->addWidget(dnsLabel);
    dnsLayout->addStretch();
    auto* dnsStatusLabel = new QLabel("Status: ");
    auto* dnsStatusValue = new QLabel("Stopped");
    dnsStatusValue->setObjectName("themeDesc");
    dnsLayout->addWidget(dnsStatusLabel);
    dnsLayout->addWidget(dnsStatusValue);
    setl->addWidget(dnsGroup);
    auto* dnsCtrlRow = new QWidget();
    auto* dnsCtrlLayout = new QHBoxLayout(dnsCtrlRow);
    dnsCtrlLayout->setContentsMargins(0,0,0,0);
    auto* dnsStartBtn = new QPushButton("\xe2\x96\xb6 Start DNS");
    dnsStartBtn->setObjectName("primary");
    dnsStartBtn->setFixedWidth(90);
    dnsStartBtn->setFixedHeight(28);
    auto* dnsStopBtn = new QPushButton("\xe2\x96\xa0 Stop DNS");
    dnsStopBtn->setObjectName("stop");
    dnsStopBtn->setFixedWidth(90);
    dnsStopBtn->setFixedHeight(28);
    dnsCtrlLayout->addWidget(dnsStartBtn);
    dnsCtrlLayout->addWidget(dnsStopBtn);
    dnsCtrlLayout->addStretch();
    setl->addWidget(dnsCtrlRow);
    auto* dnsPortRow = new QWidget();
    auto* dnsPortLayout = new QHBoxLayout(dnsPortRow);
    dnsPortLayout->setContentsMargins(0,0,0,0);
    auto* dnsPortLabel2 = new QLabel("Port: 5399");
    dnsPortLabel2->setStyleSheet("color: #60646c;");
    auto* dnsUpstreamLabel = new QLabel("Upstream: 8.8.8.8");
    dnsUpstreamLabel->setStyleSheet("color: #60646c;");
    dnsPortLayout->addWidget(dnsPortLabel2);
    dnsPortLayout->addWidget(dnsUpstreamLabel);
    dnsPortLayout->addStretch();
    setl->addWidget(dnsPortRow);
    auto* dnsDesc = new QLabel("Resolves *.test domains to 127.0.0.1. Configure in Apache vhosts.");
    dnsDesc->setObjectName("themeDesc");
    setl->addWidget(dnsDesc);
    auto refreshDnsStatus = [dnsStatusValue, dnsHealthState = (QLabel*)nullptr]() {
        bool running = boxedcat_dns_is_running();
        dnsStatusValue->setText(running ? "Running" : "Stopped");
        dnsStatusValue->setStyleSheet(running ? "color: #30a46c;" : "color: #e5484d;");
    };
    refreshDnsStatus();
    QObject::connect(dnsStartBtn, &QPushButton::clicked, [refreshDnsStatus]() {
        boxedcat_dns_start(5399, "8.8.8.8");
        refreshDnsStatus();
    });
    QObject::connect(dnsStopBtn, &QPushButton::clicked, [refreshDnsStatus]() {
        boxedcat_dns_stop();
        refreshDnsStatus();
    });
    auto* netGroup = new QWidget();
    auto* netLayout = new QHBoxLayout(netGroup);
    netLayout->setContentsMargins(0,8,0,8);
    auto* netLabel = new QLabel("Network");
    QFont nlf = netLabel->font(); nlf.setPointSize(12); nlf.setBold(true); netLabel->setFont(nlf);
    netLayout->addWidget(netLabel);
    netLayout->addStretch();
    auto* netInfo = new QLabel("Physical adapter only");
    netInfo->setStyleSheet("color: #60646c;");
    netLayout->addWidget(netInfo);
    setl->addWidget(netGroup);
    auto* netDesc = new QLabel("BoxedCat detects your physical network adapter IP. Virtual adapters (VPN, Docker, WSL) are ignored.");
    netDesc->setObjectName("themeDesc");
    setl->addWidget(netDesc);
    auto* autoGroup = new QWidget();
    auto* autoLayout = new QHBoxLayout(autoGroup);
    autoLayout->setContentsMargins(0,8,0,8);
    auto* autoLabel = new QLabel("Auto-start Services");
    QFont alf = autoLabel->font(); alf.setPointSize(12); alf.setBold(true); autoLabel->setFont(alf);
    autoLayout->addWidget(autoLabel);
    autoLayout->addStretch();
    auto* autoCheck = new QCheckBox("Enable auto-start on launch");
    autoLayout->addWidget(autoCheck);
    setl->addWidget(autoGroup);
    auto* autoDesc = new QLabel("Automatically start enabled services when BoxedCat launches.");
    autoDesc->setObjectName("themeDesc");
    setl->addWidget(autoDesc);
    auto* portGroup = new QWidget();
    auto* portLayout = new QHBoxLayout(portGroup);
    portLayout->setContentsMargins(0,8,0,8);
    auto* portLabel = new QLabel("Server Ports");
    QFont plf = portLabel->font(); plf.setPointSize(12); plf.setBold(true); portLabel->setFont(plf);
    portLayout->addWidget(portLabel);
    portLayout->addStretch();
    setl->addWidget(portGroup);
    auto* portDesc = new QLabel("HTTP: 80 | HTTPS: 443 | MariaDB: 3306 | PostgreSQL: 5432 | Redis: 6379");
    portDesc->setObjectName("themeDesc");
    setl->addWidget(portDesc);
    auto* secGroup = new QWidget();
    auto* secLayout = new QHBoxLayout(secGroup);
    secLayout->setContentsMargins(0,8,0,8);
    auto* secLabel = new QLabel("Security");
    QFont slf = secLabel->font(); slf.setPointSize(12); slf.setBold(true); secLabel->setFont(slf);
    secLayout->addWidget(secLabel);
    secLayout->addStretch();
    setl->addWidget(secGroup);
    auto* secDesc = new QLabel("All FFI inputs validated. No process spawning. Path traversal prevention enabled.");
    secDesc->setObjectName("themeDesc");
    setl->addWidget(secDesc);
    setl->addStretch();
    content->addWidget(setpg);
    auto* abtpg = new QWidget();
    auto* abl = new QVBoxLayout(abtpg);
    abl->setSpacing(12);
    abl->setContentsMargins(16,16,16,16);
    auto* iconLabel = new QLabel();
    QPixmap icon(":/icons/boxedcat_128.png");
    if (icon.isNull()) {
        QStringList iconSearch = {
            appDir + "/assets/icons/boxedcat_128.png",
            appDir + "/../assets/icons/boxedcat_128.png",
            "assets/icons/boxedcat_128.png",
        };
        for (const QString& p : iconSearch) {
            if (QFile::exists(p)) { icon.load(p); break; }
        }
    }
    if (!icon.isNull()) {
        iconLabel->setPixmap(icon.scaled(64, 64, Qt::KeepAspectRatio, Qt::SmoothTransformation));
        iconLabel->setAlignment(Qt::AlignCenter);
        abl->addWidget(iconLabel);
    }
    auto* abt = new QLabel("BoxedCat");
    QFont abf = abt->font(); abf.setPointSize(18); abf.setBold(true); abt->setFont(abf);
    abt->setAlignment(Qt::AlignCenter);
    abl->addWidget(abt);
    auto* versionLabel = new QLabel(QString("Version: %1").arg(QString::fromUtf8(boxedcat_get_version())));
    QFont vf2 = versionLabel->font(); vf2.setPointSize(13); versionLabel->setFont(vf2);
    abl->addWidget(versionLabel);
    auto* descLabel = new QLabel(
        "A standalone local dev environment manager.\n"
        "Manages Apache, MariaDB, PHP, and more.\n"
        "Built with Rust core + Qt 6 GUI.\n"
    );
    abl->addWidget(descLabel);
    auto* featuresTitle = new QLabel("Features");
    QFont fft = featuresTitle->font(); fft.setPointSize(12); fft.setBold(true); featuresTitle->setFont(fft);
    abl->addWidget(featuresTitle);
    auto* featuresList = new QLabel(
        "\xe2\x80\xa2  Service management (Apache, MariaDB)\n"
        "\xe2\x80\xa2  Port succession with fallback\n"
        "\xe2\x80\xa2  Virtual host generation (*.test)\n"
        "\xe2\x80\xa2  DNS proxy (*.test \xe2\x86\x92 127.0.0.1)\n"
        "\xe2\x80\xa2  Live port scanning (native Windows API)\n"
        "\xe2\x80\xa2  Project auto-detection from www/\n"
        "\xe2\x80\xa2  Radix UI adaptive theme (light/dark)"
    );
    abl->addWidget(featuresList);
    auto* techTitle = new QLabel("Technology");
    QFont ftt = techTitle->font(); ftt.setPointSize(12); ftt.setBold(true); techTitle->setFont(ftt);
    abl->addWidget(techTitle);
    auto* techList = new QLabel(
        "\xe2\x80\xa2  Rust 1.x (core logic, FFI exports)\n"
        "\xe2\x80\xa2  Qt 6.7.3 (GUI framework)\n"
        "\xe2\x80\xa2  MinGW 11.2.0 (ABI-compatible compiler)\n"
        "\xe2\x80\xa2  Windows native APIs (no external process spawning)"
    );
    abl->addWidget(techList);
    abl->addStretch();
    content->addWidget(abtpg);
    splitter->addWidget(content);
    splitter->setStretchFactor(1,1);
    mainLayout->addWidget(splitter);
    w.setCentralWidget(central);
    auto* statusBar = new QStatusBar();
    w.setStatusBar(statusBar);
    statusBar->showMessage("Ready");
    QObject::connect(nav, &QListWidget::currentRowChanged, [content, refreshDnsHealth, refreshDnsStatus](int row) {
        content->setCurrentIndex(row);
        if (row == 0) refreshDnsHealth();
        if (row == 5) refreshDnsStatus();
    });
    QObject::connect(lightBtn, &QPushButton::clicked, [themeHint]() {
        saveDarkMode(false);
        applyTheme(false);
        themeHint->setText("Current: Light");
    });
    QObject::connect(darkBtn, &QPushButton::clicked, [themeHint]() {
        saveDarkMode(true);
        applyTheme(true);
        themeHint->setText("Current: Dark");
    });
    BcServiceList services = boxedcat_get_services();
    for (unsigned long i = 0; i < services.count; i++) {
        const BcServiceInfo& s = services.items[i];
        QString port = QString::number(s.port);
        int row = dashTable->rowCount();
        dashTable->insertRow(row);
        dashTable->setItem(row, 0, new QTableWidgetItem(s.display_name));
        auto* statusItem = new QTableWidgetItem(s.status);
        statusItem->setTextAlignment(Qt::AlignCenter);
        dashTable->setItem(row, 1, statusItem);
        auto* portItem = new QTableWidgetItem(port.toUtf8().constData());
        portItem->setTextAlignment(Qt::AlignCenter);
        dashTable->setItem(row, 2, portItem);
        auto* pidItem = new QTableWidgetItem("-");
        pidItem->setTextAlignment(Qt::AlignCenter);
        dashTable->setItem(row, 3, pidItem);
        auto* healthItem = new QTableWidgetItem(QString::fromUtf8(s.status));
        healthItem->setForeground(QString::fromUtf8(s.status) == "Running" ? QColor("#30a46c") : QColor("#e5484d"));
        healthItem->setTextAlignment(Qt::AlignCenter);
        dashTable->setItem(row, 4, healthItem);
        dashTable->setRowHeight(row, 36);
    }
    ServiceContext svcCtx[3];
    for (unsigned long i = 0; i < services.count; i++) {
        const BcServiceInfo& s = services.items[i];
        QString port = QString::number(s.port);
        int row = svcTable->rowCount();
        svcTable->insertRow(row);
        svcTable->setItem(row, 0, new QTableWidgetItem(s.display_name));
        auto* svcStatusItem = new QTableWidgetItem(s.status);
        svcStatusItem->setTextAlignment(Qt::AlignCenter);
        svcTable->setItem(row, 1, svcStatusItem);
        auto* svcPortItem = new QTableWidgetItem(port.toUtf8().constData());
        svcPortItem->setTextAlignment(Qt::AlignCenter);
        svcTable->setItem(row, 2, svcPortItem);
        auto* svcVersionItem = new QTableWidgetItem(s.version);
        svcVersionItem->setTextAlignment(Qt::AlignCenter);
        svcTable->setItem(row, 3, svcVersionItem);
        auto* ctrlWidget = new QWidget();
        auto* ctrlLayout = new QHBoxLayout(ctrlWidget);
        ctrlLayout->setContentsMargins(2,2,2,2);
        ctrlLayout->setSpacing(4);
        ctrlLayout->setAlignment(Qt::AlignCenter);
        auto* startBtn = new QPushButton("\xe2\x96\xb6 Start");
        startBtn->setObjectName("start");
        startBtn->setFixedWidth(80);
        startBtn->setFixedHeight(28);
        auto* stopBtn = new QPushButton("\xe2\x96\xa0 Stop");
        stopBtn->setObjectName("stop");
        stopBtn->setFixedWidth(80);
        stopBtn->setFixedHeight(28);
        auto* restartBtn = new QPushButton("\xe2\x86\xbb Restart");
        restartBtn->setObjectName("restart");
        restartBtn->setFixedWidth(90);
        restartBtn->setFixedHeight(28);
        ctrlLayout->addWidget(startBtn);
        ctrlLayout->addWidget(stopBtn);
        ctrlLayout->addWidget(restartBtn);
        svcTable->setRowHeight(row, 40);
        svcTable->setCellWidget(row, 4, ctrlWidget);
        svcCtx[i].name = s.name;
        svcCtx[i].table = svcTable;
        svcCtx[i].row = row;
        QObject::connect(startBtn, &QPushButton::clicked, [svcCtx, i]() {
            bool ok = boxedcat_service_start(svcCtx[i].name);
            if (ok) refreshServiceStatus(svcCtx[i]);
        });
        QObject::connect(stopBtn, &QPushButton::clicked, [svcCtx, i]() {
            bool ok = boxedcat_service_stop(svcCtx[i].name);
            if (ok) refreshServiceStatus(svcCtx[i]);
        });
        QObject::connect(restartBtn, &QPushButton::clicked, [svcCtx, i]() {
            boxedcat_service_stop(svcCtx[i].name);
            bool ok = boxedcat_service_start(svcCtx[i].name);
            if (ok) refreshServiceStatus(svcCtx[i]);
        });
    }
    unsigned long svcCount = services.count;
    QObject::connect(refreshAllBtn, &QPushButton::clicked, [&svcCtx, svcCount, dashTable, svcTable, services]() {
        for (unsigned long i = 0; i < svcCount; i++) {
            const char* status = boxedcat_refresh_service(svcCtx[i].name);
            svcTable->setItem(svcCtx[i].row, 1, new QTableWidgetItem(status));
            dashTable->setItem(i, 1, new QTableWidgetItem(status));
            auto* healthItem = new QTableWidgetItem(QString::fromUtf8(status));
            healthItem->setForeground(QString::fromUtf8(status) == "Running" ? QColor("#30a46c") : QColor("#e5484d"));
            dashTable->setItem(i, 4, healthItem);
        }
    });
    w.show();
    if (splash) {
        splash->finish(&w);
        delete splash;
    }
    int result = app.exec();
    boxedcat_shutdown();
    return result;
}
