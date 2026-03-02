// ─── iOS Project Structure (Swift + SwiftUI) ───
//
// CarDeal/
// ├── CarDealApp.swift              ← @main entry + TabView
// ├── Info.plist
// ├── DI/
// │   └── Dependencies.swift        ← Service container
// ├── Data/
// │   ├── API/
// │   │   ├── CarDealAPI.swift      ← URLSession / Alamofire
// │   │   └── APIClient.swift       ← Base client + interceptors
// │   ├── Models/
// │   │   └── VehicleReport.swift   ← (already created)
// │   └── Repository/
// │       └── VehicleRepository.swift
// ├── UI/
// │   ├── Theme/
// │   │   ├── AppTheme.swift
// │   │   ├── Colors.swift
// │   │   └── Typography.swift
// │   ├── Screens/
// │   │   ├── Splash/
// │   │   │   └── SplashView.swift
// │   │   ├── Home/
// │   │   │   ├── HomeView.swift
// │   │   │   └── HomeViewModel.swift
// │   │   ├── Report/
// │   │   │   ├── ReportView.swift
// │   │   │   ├── ReportViewModel.swift
// │   │   │   └── Sections/
// │   │   │       ├── VehicleOverviewCard.swift
// │   │   │       ├── SpecsCard.swift
// │   │   │       ├── OwnershipCard.swift
// │   │   │       ├── InspectionCard.swift
// │   │   │       ├── RecallsCard.swift
// │   │   │       ├── TaxCostsCard.swift
// │   │   │       ├── PriceEstimateCard.swift
// │   │   │       ├── RiskCard.swift
// │   │   │       ├── ConditionCard.swift
// │   │   │       ├── SuitabilityCard.swift
// │   │   │       └── AiSummaryCard.swift
// │   │   ├── Plans/
// │   │   │   ├── PlansView.swift
// │   │   │   └── PlansViewModel.swift
// │   │   └── Profile/
// │   │       ├── ProfileView.swift
// │   │       └── ProfileViewModel.swift
// │   ├── Components/
// │   │   ├── SearchBar.swift
// │   │   ├── ReportCard.swift
// │   │   ├── ScoreGauge.swift
// │   │   ├── PriceRangeBar.swift
// │   │   ├── TimelineView.swift
// │   │   ├── AlertBadge.swift
// │   │   ├── CreditBadge.swift
// │   │   └── LoadingOverlay.swift
// │   └── Navigation/
// │       └── TabNavigation.swift
// └── Util/
//     ├── LocaleManager.swift
//     └── DateFormatter.swift
